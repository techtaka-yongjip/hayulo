from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.benchmarks import LLM_BENCHMARK_SCHEMA, LLM_BENCHMARK_TASK_SCHEMA, load_llm_benchmark_tasks


ROOT = Path(__file__).resolve().parents[1]


class LlmBenchmarkTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_benchmark_cli_json_summary(self):
        result = self.run_cli("benchmark", "llm", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["schema"], LLM_BENCHMARK_SCHEMA)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["kind"], "llm-benchmark-suite")
        self.assertEqual(payload["summary"]["tasks"], 5)
        self.assertEqual(payload["summary"]["recorded_runs"], 0)
        self.assertEqual(payload["summary"]["comparison_targets"]["hayulo"], 5)
        self.assertEqual(payload["summary"]["comparison_targets"]["python-fastapi"], 5)
        self.assertEqual(payload["summary"]["comparison_targets"]["typescript-fastify"], 5)
        self.assertEqual(payload["summary"]["comparison_targets"]["go-stdlib"], 5)

    def test_task_fixtures_are_valid(self):
        tasks = load_llm_benchmark_tasks(ROOT)
        self.assertEqual(len(tasks), 5)
        ids = {task.id for task in tasks}
        self.assertEqual(
            ids,
            {
                "api.todo_crud",
                "api.inventory",
                "api.webhook_receiver",
                "api.support_ticket",
                "api.reading_list",
            },
        )
        for task in tasks:
            with self.subTest(task=task.id):
                task_json = json.loads(task.source_file.read_text(encoding="utf-8"))
                self.assertEqual(task_json["schema"], LLM_BENCHMARK_TASK_SCHEMA)
                self.assertTrue(task.hayulo_baseline.is_file())
                self.assertIn("hayulo", task.comparison_targets)
                self.assertIn("repair_iterations", task.success_metrics)
                self.assertTrue(task.manual_checks)

    def test_hayulo_baselines_pass_project_check(self):
        result = self.run_cli("check", "benchmarks/llm/baselines", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "ok")
        self.assertEqual(payload["kind"], "project")
        self.assertEqual(payload["checked"], 5)
        self.assertTrue(all(file["kind"] == "api" for file in payload["files"]))


class LlmBenchmarkGeneratedApiTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_hayulo_baselines_build_generated_apis(self):
        tasks = load_llm_benchmark_tasks(ROOT)
        with tempfile.TemporaryDirectory() as tmp:
            for task in tasks:
                with self.subTest(task=task.id):
                    out = Path(tmp) / task.id.replace(".", "_")
                    baseline = task.hayulo_baseline.relative_to(ROOT)
                    result = self.run_cli("build", str(baseline), "--out", str(out), "--json")
                    self.assertEqual(result.returncode, 0, result.stderr)
                    payload = json.loads(result.stdout)
                    self.assertEqual(payload["status"], "ok")
                    self.assertEqual(payload["kind"], "api-build")
                    generated = {Path(item["path"]).name for item in payload["generated"]}
                    self.assertEqual(generated, {"hayulo.ir.json", "openapi.json", "package.json", "server.mjs", "smoke_test.mjs", "README.md"})

    @unittest.skipUnless(shutil.which("node") is not None, "node is required for generated benchmark API smoke tests")
    def test_hayulo_baselines_generated_smoke_tests_pass(self):
        tasks = load_llm_benchmark_tasks(ROOT)
        with tempfile.TemporaryDirectory() as tmp:
            for task in tasks:
                with self.subTest(task=task.id):
                    out = Path(tmp) / task.id.replace(".", "_")
                    baseline = task.hayulo_baseline.relative_to(ROOT)
                    build = self.run_cli("build", str(baseline), "--out", str(out), "--json")
                    self.assertEqual(build.returncode, 0, build.stderr)
                    smoke = subprocess.run(
                        ["npm", "test"],
                        cwd=out,
                        text=True,
                        capture_output=True,
                        check=False,
                        timeout=20,
                    )
                    self.assertEqual(smoke.returncode, 0, smoke.stderr)
                    self.assertIn("smoke test passed", smoke.stdout.lower())


if __name__ == "__main__":
    unittest.main()
