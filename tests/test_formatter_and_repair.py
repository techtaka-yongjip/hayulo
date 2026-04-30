from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.formatter import format_source


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures"
REPAIR_FIXTURES = ROOT / "tests" / "repair_fixtures"


class FormatterAndRepairCliTests(unittest.TestCase):
    def run_cli(self, *args: str, cwd: Path | None = None):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=cwd or ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_formatter_golden_output(self):
        source = (FIXTURES / "unformatted.hayulo").read_text(encoding="utf-8")
        expected = (FIXTURES / "formatted.hayulo").read_text(encoding="utf-8")
        self.assertEqual(format_source(source), expected)
        self.assertEqual(format_source(expected), expected)

    def test_format_check_passes_for_formatted_file(self):
        result = self.run_cli("format", "--check", str(FIXTURES / "formatted.hayulo"), "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["mode"], "check")
        self.assertEqual(payload["changed"], 0)

    def test_format_check_reports_unformatted_file_without_rewriting(self):
        path = FIXTURES / "unformatted.hayulo"
        before = path.read_text(encoding="utf-8")
        result = self.run_cli("format", "--check", str(path), "--json")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "")
        self.assertEqual(path.read_text(encoding="utf-8"), before)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["schema"], "hayulo.diagnostics@0.1")
        self.assertEqual(payload["diagnostics"][0]["code"], "format.required")
        self.assertEqual(payload["errors"][0]["code"], "format.required")

    def test_format_rewrites_temp_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sample.hayulo"
            path.write_text((FIXTURES / "unformatted.hayulo").read_text(encoding="utf-8"), encoding="utf-8")
            result = self.run_cli("format", str(path), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["changed"], 1)
            self.assertEqual(path.read_text(encoding="utf-8"), (FIXTURES / "formatted.hayulo").read_text(encoding="utf-8"))

    def test_summarize_json_project(self):
        result = self.run_cli("summarize", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["kind"], "project-summary")
        self.assertGreaterEqual(payload["totals"]["files"], 3)
        self.assertGreaterEqual(payload["totals"]["functions"], 1)
        self.assertGreaterEqual(payload["totals"]["routes"], 4)
        self.assertTrue(any(file["file"] == "examples/todo_api/main.hayulo" for file in payload["files"]))

    def test_failing_test_json_schema(self):
        result = self.run_cli("test", str(FIXTURES / "failing_test.hayulo"), "--json")
        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["schema"], "hayulo.test@0.1")
        self.assertEqual(payload["summary"], {"passed": 0, "failed": 1})
        self.assertEqual(payload["failures"][0]["test"], "failure shape")
        self.assertEqual(payload["failures"][0]["message"], "Expectation failed.")

    def test_repair_fixtures_emit_stable_diagnostics(self):
        expected = {
            "unknown_name.hayulo": "name.unknown_symbol",
            "bad_arity.hayulo": "call.arity_mismatch",
            "bad_record_field.hayulo": "record.unknown_field",
        }
        for filename, code in expected.items():
            with self.subTest(filename=filename):
                result = self.run_cli("check", str(REPAIR_FIXTURES / filename), "--json")
                self.assertEqual(result.returncode, 1)
                payload = json.loads(result.stdout)
                self.assertEqual(payload["schema"], "hayulo.diagnostics@0.1")
                self.assertEqual(payload["diagnostics"][0]["code"], code)
                self.assertEqual(payload["errors"][0]["code"], code)


if __name__ == "__main__":
    unittest.main()
