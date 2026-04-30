from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.api import generate_api, parse_api_source

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples" / "todo_api" / "main.hayulo"


class ApiBuildTests(unittest.TestCase):
    def run_cli(self, *args: str, cwd: Path | None = None):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=cwd or ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_parse_todo_api(self):
        spec = parse_api_source(EXAMPLE.read_text(encoding="utf-8"), filename=str(EXAMPLE))
        self.assertEqual(spec.app_name, "TodoApi")
        self.assertIn("Todo", spec.records)
        self.assertIn("CreateTodo", spec.records)
        self.assertEqual(len(spec.routes), 4)
        self.assertEqual(spec.routes[0].method, "GET")
        self.assertEqual(spec.routes[1].body_type, "CreateTodo")
        self.assertEqual(spec.routes[0].effects, ["api.read", "storage.local"])
        self.assertEqual(spec.routes[0].action.kind, "list")
        self.assertEqual(spec.routes[1].action.kind, "create")
        self.assertEqual(spec.records["Todo"].fields[1].constraints, {"min": 1, "max": 200})

    def test_generate_api_files(self):
        spec = parse_api_source(EXAMPLE.read_text(encoding="utf-8"), filename=str(EXAMPLE))
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "generated"
            files = generate_api(spec, out)
            names = {file.path.name for file in files}
            self.assertIn("server.mjs", names)
            self.assertIn("openapi.json", names)
            self.assertIn("hayulo.ir.json", names)
            openapi = json.loads((out / "openapi.json").read_text(encoding="utf-8"))
            self.assertEqual(openapi["info"]["title"], "Todo API")
            self.assertIn("/health", openapi["paths"])
            self.assertIn("/todos", openapi["paths"])
            self.assertEqual(openapi["paths"]["/todos"]["post"]["responses"]["201"]["description"], "Created")
            self.assertIn("400", openapi["paths"]["/todos"]["post"]["responses"])
            self.assertEqual(openapi["paths"]["/todos/{id}"]["delete"]["responses"]["204"]["description"], "Deleted")
            self.assertIn("ErrorResponse", openapi["components"]["schemas"])
            ir = json.loads((out / "hayulo.ir.json").read_text(encoding="utf-8"))
            self.assertEqual(ir["routes"][0]["action"], "list")
            self.assertEqual(ir["routes"][1]["action"], "create")
            self.assertEqual(ir["routes"][2]["updates"], {"done": True})

    def test_cli_build_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.run_cli("build", str(EXAMPLE), "--out", str(Path(tmp) / "generated"), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "ok")
            self.assertEqual(payload["app"], "TodoApi")

    def test_old_inline_constraints_are_rejected(self):
        source = """
module old_api
app OldApi {
  type Todo = record {
    id: Id<Todo>
    title: Text min 1 max 200
  }
  route GET "/todos" -> List<Todo> {
    effect api.read
    effect storage.local
    action list Todo
  }
}
"""
        with self.assertRaises(Exception) as context:
            parse_api_source(source, filename="<test>")
        self.assertEqual(context.exception.diagnostic.code, "api.inline_constraints_removed")

    def test_old_db_route_body_is_rejected(self):
        source = """
module old_api
app OldApi {
  type Todo = record {
    id: Id<Todo>
    title: Text { min: 1, max: 200 }
  }
  route GET "/todos" -> List<Todo> {
    return db.Todo.all()
  }
}
"""
        with self.assertRaises(Exception) as context:
            parse_api_source(source, filename="<test>")
        self.assertEqual(context.exception.diagnostic.code, "route.body_requires_action")

    def test_old_db_assignment_route_body_is_rejected(self):
        source = """
module old_api
app OldApi {
  type Todo = record {
    id: Id<Todo>
    title: Text { min: 1, max: 200 }
  }
  route PATCH "/todos/{id}" -> Todo {
    todo = db.Todo.get(id)
    return db.Todo.update(todo)
  }
}
"""
        with self.assertRaises(Exception) as context:
            parse_api_source(source, filename="<test>")
        self.assertEqual(context.exception.diagnostic.code, "route.body_requires_action")

    def test_update_action_source_must_match_body_binding(self):
        source = """
module bad_api
app BadApi {
  type Todo = record {
    id: Id<Todo>
    title: Text { min: 1, max: 200 }
  }
  type UpdateTodo = record {
    title: Text { min: 1, max: 200 }
  }
  route PATCH "/todos/{id}" body input: UpdateTodo -> Todo {
    effect api.write
    effect storage.local
    action update Todo by id from payload
  }
}
"""
        with self.assertRaises(Exception) as context:
            parse_api_source(source, filename="<test>")
        self.assertEqual(context.exception.diagnostic.code, "route.action_body_mismatch")

    def test_new_api_project_can_check_and_build(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "todo-service"
            new_result = self.run_cli("new", "api", str(project), "--json")
            self.assertEqual(new_result.returncode, 0, new_result.stderr)
            new_payload = json.loads(new_result.stdout)
            self.assertEqual(new_payload["kind"], "project-new-api")
            self.assertEqual(new_payload["template"], "api")
            self.assertTrue((project / "hayulo.toml").is_file())
            self.assertTrue((project / "src" / "main.hayulo").is_file())

            check_result = self.run_cli("check", "--json", cwd=project)
            self.assertEqual(check_result.returncode, 0, check_result.stderr)
            check_payload = json.loads(check_result.stdout)
            self.assertEqual(check_payload["kind"], "project")
            self.assertEqual(check_payload["files"][0]["kind"], "api")

            build_result = self.run_cli("build", "src/main.hayulo", "--json", cwd=project)
            self.assertEqual(build_result.returncode, 0, build_result.stderr)
            build_payload = json.loads(build_result.stdout)
            self.assertEqual(build_payload["kind"], "api-build")
            self.assertTrue((project / "src" / "generated" / "openapi.json").is_file())

            openapi = json.loads((project / "src" / "generated" / "openapi.json").read_text(encoding="utf-8"))
            self.assertIn("/todos/{id}", openapi["paths"])
            parameter = openapi["paths"]["/todos/{id}"]["get"]["parameters"][0]
            self.assertEqual(parameter["name"], "id")
            self.assertEqual(parameter["schema"], {"type": "integer"})

            if shutil.which("node") is not None:
                smoke = subprocess.run(
                    ["npm", "test"],
                    cwd=project / "src" / "generated",
                    text=True,
                    capture_output=True,
                    check=False,
                    timeout=20,
                )
                self.assertEqual(smoke.returncode, 0, smoke.stderr)
                self.assertIn("smoke test passed", smoke.stdout.lower())

    @unittest.skipUnless(os.environ.get("HAYULO_RUN_NODE_TESTS") == "1" and shutil.which("node") is not None, "set HAYULO_RUN_NODE_TESTS=1 with node installed")
    def test_generated_smoke_test_runs_when_node_available(self):
        spec = parse_api_source(EXAMPLE.read_text(encoding="utf-8"), filename=str(EXAMPLE))
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "generated"
            generate_api(spec, out)
            result = subprocess.run(
                ["node", "smoke_test.mjs"],
                cwd=out,
                text=True,
                capture_output=True,
                check=False,
                timeout=20,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("smoke test passed", result.stdout)


if __name__ == "__main__":
    unittest.main()
