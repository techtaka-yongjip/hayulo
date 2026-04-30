from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.project import read_project_config


ROOT = Path(__file__).resolve().parents[1]


API_SOURCE = """module denied_api

app DeniedApi {
  database sqlite "todo.db"

  openapi {
    title: "Denied API"
    version: "0.1.0"
  }

  type Todo = record {
    id: Id<Todo>
    title: Text min 1 max 200
  }

  route GET "/todos" -> List<Todo> {
    return db.Todo.all()
  }

  route POST "/todos" body input: CreateTodo -> Todo {
    return db.Todo.insert(Todo { title: input.title })
  }

  route DELETE "/todos/{id}" -> Status {
    db.Todo.delete(id)?
    return no_content
  }
}

type CreateTodo = record {
  title: Text min 1 max 200
}
"""


class PermissionTests(unittest.TestCase):
    def run_cli(self, *args: str, cwd: Path | None = None):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=cwd or ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def write_project(self, root: Path, *, allow: list[str], deny: list[str]) -> Path:
        (root / "src").mkdir(parents=True)
        (root / "hayulo.toml").write_text(
            f"""[project]
name = "permission-test"
version = "0.1.0"
src = "src"
tests = "src"

[permissions]
allow = {json.dumps(allow)}
deny = {json.dumps(deny)}
""",
            encoding="utf-8",
        )
        source = root / "src" / "main.hayulo"
        source.write_text(API_SOURCE, encoding="utf-8")
        return source

    def test_permission_parser_reads_allow_and_deny(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_project(root, allow=["api.read", "storage.local"], deny=["api.delete"])
            config = read_project_config(root)
            self.assertEqual(config.permissions.allow, frozenset({"api.read", "storage.local"}))
            self.assertEqual(config.permissions.deny, frozenset({"api.delete"}))

    def test_invalid_permission_name_is_diagnostic(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_project(root, allow=["API.READ"], deny=[])
            result = self.run_cli("check", "--json", cwd=root)
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["diagnostics"][0]["code"], "project.invalid_permission")
            self.assertEqual(payload["errors"][0]["code"], "project.invalid_permission")

    def test_check_reports_missing_permission(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_project(root, allow=["api.read", "api.write", "storage.local"], deny=[])
            result = self.run_cli("check", "--json", cwd=root)
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["diagnostics"][0]["code"], "permission.missing")
            self.assertEqual(payload["diagnostics"][0]["details"]["permission"], "api.delete")
            self.assertEqual(payload["errors"][0]["code"], "permission.missing")

    def test_build_reports_denied_permission(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.write_project(root, allow=["api.read", "api.write", "api.delete", "storage.local"], deny=["api.delete"])
            result = self.run_cli("build", str(source), "--json", cwd=root)
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["diagnostics"][0]["code"], "permission.denied")
            self.assertEqual(payload["diagnostics"][0]["details"]["permission"], "api.delete")
            self.assertEqual(payload["errors"][0]["code"], "permission.denied")


if __name__ == "__main__":
    unittest.main()
