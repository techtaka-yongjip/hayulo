from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class ProjectSystemCliTests(unittest.TestCase):
    def run_cli(self, *args: str, cwd: Path | None = None):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=cwd or ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_new_creates_project_layout(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample-app"
            result = self.run_cli("new", str(project), "--json")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "ok")
            self.assertEqual(payload["kind"], "project-new")
            self.assertTrue((project / "hayulo.toml").is_file())
            self.assertTrue((project / "src" / "main.hayulo").is_file())
            self.assertTrue((project / "tests" / "main_test.hayulo").is_file())

    def test_new_refuses_non_empty_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample-app"
            project.mkdir()
            (project / "README.md").write_text("# exists\n", encoding="utf-8")
            result = self.run_cli("new", str(project), "--json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["errors"][0]["code"], "project.exists")

    def test_project_check_and_test_generated_project(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample-app"
            self.assertEqual(self.run_cli("new", str(project)).returncode, 0)

            check_result = self.run_cli("check", "--json", cwd=project)
            self.assertEqual(check_result.returncode, 0, check_result.stderr)
            check_payload = json.loads(check_result.stdout)
            self.assertEqual(check_payload["kind"], "project")
            self.assertEqual(check_payload["checked"], 2)
            self.assertEqual([item["file"] for item in check_payload["files"]], ["src/main.hayulo", "tests/main_test.hayulo"])

            test_result = self.run_cli("test", "--json", cwd=project)
            self.assertEqual(test_result.returncode, 0, test_result.stderr)
            test_payload = json.loads(test_result.stdout)
            self.assertEqual(test_payload["kind"], "project-test")
            self.assertEqual(test_payload["passed"], 1)
            self.assertEqual(test_payload["failed"], 0)

    def test_project_check_discovers_config_from_subdirectory(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp) / "sample-app"
            self.assertEqual(self.run_cli("new", str(project)).returncode, 0)
            result = self.run_cli("check", "--json", cwd=project / "src")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["project"]["name"], "sample-app")

    def test_single_file_check_still_works(self):
        result = self.run_cli("check", str(ROOT / "examples" / "hello.hayulo"), "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["kind"], "script")
        self.assertEqual(payload["module"], "hello")


if __name__ == "__main__":
    unittest.main()
