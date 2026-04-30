from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo import __version__
from hayulo.diagnostics import DIAGNOSTIC_SCHEMA, TEST_SCHEMA


ROOT = Path(__file__).resolve().parents[1]


class StableContractTests(unittest.TestCase):
    def run_cli(self, *args: str, cwd: Path | None = None):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=cwd or ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_version_contract(self):
        self.assertEqual(__version__, "1.0.0")
        result = self.run_cli("--version")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "hayulo 1.0.0\n")

    def test_stable_help_lists_public_commands(self):
        result = self.run_cli("--help")
        self.assertEqual(result.returncode, 0, result.stderr)
        for command in ["check", "run", "test", "format", "summarize", "new", "build"]:
            self.assertIn(command, result.stdout)

    def test_diagnostic_schema_contract(self):
        self.assertEqual(DIAGNOSTIC_SCHEMA, "hayulo.diagnostics@0.1")
        result = self.run_cli("check", "tests/fixtures/missing_file.hayulo", "--json")
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["schema"], DIAGNOSTIC_SCHEMA)
        self.assertEqual(payload["diagnostics"][0]["code"], "file_not_found")
        self.assertIn("errors", payload)

    def test_test_schema_contract(self):
        self.assertEqual(TEST_SCHEMA, "hayulo.test@0.1")
        result = self.run_cli("test", "tests/fixtures/failing_test.hayulo", "--json")
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["schema"], TEST_SCHEMA)
        self.assertEqual(payload["summary"], {"passed": 0, "failed": 1})
        self.assertIn("passed", payload)
        self.assertIn("failed", payload)

    def test_formatter_check_contract(self):
        formatted = self.run_cli("format", "--check", "tests/fixtures/formatted.hayulo", "--json")
        self.assertEqual(formatted.returncode, 0, formatted.stderr)
        formatted_payload = json.loads(formatted.stdout)
        self.assertEqual(formatted_payload["kind"], "format")
        self.assertEqual(formatted_payload["mode"], "check")
        self.assertEqual(formatted_payload["changed"], 0)

        unformatted = self.run_cli("format", "--check", "tests/fixtures/unformatted.hayulo", "--json")
        self.assertEqual(unformatted.returncode, 1)
        unformatted_payload = json.loads(unformatted.stdout)
        self.assertEqual(unformatted_payload["diagnostics"][0]["code"], "format.required")

    def test_summarize_contract(self):
        result = self.run_cli("summarize", "examples/todo_api/main.hayulo", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["kind"], "api-summary")
        self.assertEqual(payload["app"], "TodoApi")
        self.assertGreaterEqual(len(payload["routes"]), 4)

    def test_api_build_contract(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.run_cli("build", "examples/todo_api/main.hayulo", "--out", str(Path(tmp) / "generated"), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["kind"], "api-build")
            self.assertEqual(payload["permissions"]["required"], ["api.delete", "api.read", "api.write", "storage.local"])
            generated = {Path(item["path"]).name for item in payload["generated"]}
            self.assertEqual(generated, {"hayulo.ir.json", "openapi.json", "package.json", "server.mjs", "smoke_test.mjs", "README.md"})

    def test_stable_docs_exist(self):
        for path in [
            "docs/stable_contract_1_0.md",
            "docs/compatibility.md",
            "docs/migration_policy.md",
            "docs/standard_library_core.md",
            "docs/release_checklist.md",
        ]:
            self.assertTrue((ROOT / path).is_file(), path)


if __name__ == "__main__":
    unittest.main()
