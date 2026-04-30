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
            self.assertIn("/todos", openapi["paths"])

    def test_cli_build_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run(
                [sys.executable, "-m", "hayulo", "build", str(EXAMPLE), "--out", str(Path(tmp) / "generated"), "--json"],
                cwd=ROOT,
                env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "ok")
            self.assertEqual(payload["app"], "TodoApi")

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
