from __future__ import annotations

import json
import re
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"


class PublicAlphaDocsTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_public_alpha_docs_are_linked(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        index = (DOCS / "INDEX.md").read_text(encoding="utf-8")
        for path in [
            "docs/public_alpha.md",
            "docs/syntax_subset.md",
            "docs/syntax_rulebook.md",
            "docs/repair_benchmarks.md",
            "docs/ci.md",
            "docs/editor_support.md",
        ]:
            self.assertIn(path, readme)
            self.assertTrue((ROOT / path).is_file())
        for path in [
            "public_alpha.md",
            "syntax_subset.md",
            "syntax_rulebook.md",
            "repair_benchmarks.md",
            "ci.md",
            "editor_support.md",
        ]:
            self.assertIn(path, index)

    def test_public_alpha_doc_links_resolve(self):
        for doc in DOCS.glob("*.md"):
            text = doc.read_text(encoding="utf-8")
            for match in re.finditer(r"\[[^\]]+\]\(([^)]+)\)", text):
                target = match.group(1)
                if "://" in target or target.startswith("#"):
                    continue
                target = target.split("#", 1)[0]
                if not target:
                    continue
                self.assertTrue((doc.parent / target).resolve().exists(), f"{doc.name} links to missing {target}")

    def test_editor_grammar_is_valid_json(self):
        grammar = json.loads((ROOT / "editors" / "hayulo.tmLanguage.json").read_text(encoding="utf-8"))
        self.assertEqual(grammar["scopeName"], "source.hayulo")
        self.assertIn("hayulo", grammar["fileTypes"])
        self.assertIn("repository", grammar)

    def test_ci_workflow_contains_public_alpha_gate(self):
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        self.assertIn("actions/setup-python@v5", workflow)
        self.assertIn("actions/setup-node@v4", workflow)
        self.assertIn("make test", workflow)
        self.assertIn("make check", workflow)
        self.assertIn("make release-check", workflow)

    def test_repair_benchmark_document_matches_fixtures(self):
        doc = (DOCS / "repair_benchmarks.md").read_text(encoding="utf-8")
        expected = {
            "tests/repair_fixtures/unknown_name.hayulo": "name.unknown_symbol",
            "tests/repair_fixtures/bad_arity.hayulo": "call.arity_mismatch",
            "tests/repair_fixtures/bad_record_field.hayulo": "record.unknown_field",
        }
        for path, code in expected.items():
            self.assertIn(path, doc)
            self.assertIn(code, doc)
            result = self.run_cli("check", path, "--json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["diagnostics"][0]["code"], code)

    def test_candidate_subset_marks_1_0_and_limits(self):
        text = (DOCS / "syntax_subset.md").read_text(encoding="utf-8")
        self.assertIn("Hayulo 1.0 stable syntax subset", text)
        self.assertIn("Out of Scope for 1.0", text)
        self.assertIn("TypeScript generation", text)
        self.assertIn("language-level effects enforcement", text)

    def test_syntax_rulebook_is_canonical_and_strict(self):
        text = (DOCS / "syntax_rulebook.md").read_text(encoding="utf-8")
        for phrase in [
            "canonical rulebook",
            "one official shape",
            "No Hidden Control Flow",
            "No Implicit Dangerous Values",
            "Diagnostics Are Part Of The Syntax",
            "The Formatter Decides Style",
            "Benchmark Before Preference",
            "Feature Proposal Checklist",
            "Rejection Checklist",
        ]:
            self.assertIn(phrase, text)

        principles = (DOCS / "language_design_principles.md").read_text(encoding="utf-8")
        duplicate_principles = (DOCS / "design_principles.md").read_text(encoding="utf-8")
        self.assertIn("syntax_rulebook.md", principles)
        self.assertIn("syntax_rulebook.md", duplicate_principles)


if __name__ == "__main__":
    unittest.main()
