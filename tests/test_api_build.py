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

    def test_cli_build_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.run_cli("build", str(EXAMPLE), "--out", str(Path(tmp) / "generated"), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "ok")
            self.assertEqual(payload["app"], "TodoApi")

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
