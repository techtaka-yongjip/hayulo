from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SNAPSHOTS = ROOT / "tests" / "snapshots"


class CliSnapshotTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def assert_json_snapshot(self, snapshot_name: str, result: subprocess.CompletedProcess[str]) -> None:
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "")
        self.assertNotIn("Traceback", result.stdout)
        payload = json.loads(result.stdout)
        actual = json.dumps(payload, indent=2, sort_keys=True) + "\n"
        expected = (SNAPSHOTS / snapshot_name).read_text(encoding="utf-8")
        self.assertEqual(actual, expected)

    def test_script_syntax_error_snapshot(self):
        result = self.run_cli("check", "tests/fixtures/syntax_error.hayulo", "--json")
        self.assert_json_snapshot("script_syntax_error.json", result)

    def test_missing_file_snapshot(self):
        result = self.run_cli("check", "tests/fixtures/missing_file.hayulo", "--json")
        self.assert_json_snapshot("missing_file.json", result)

    def test_runtime_error_snapshot(self):
        result = self.run_cli("run", "tests/fixtures/runtime_error.hayulo", "--json")
        self.assert_json_snapshot("runtime_error.json", result)

    def test_api_build_error_snapshot(self):
        result = self.run_cli("build", "tests/fixtures/api_error.hayulo", "--json")
        self.assert_json_snapshot("api_build_error.json", result)


if __name__ == "__main__":
    unittest.main()
