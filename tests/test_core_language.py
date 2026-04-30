from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

from hayulo.interpreter import Interpreter
from hayulo.lexer import Lexer
from hayulo.parser import Parser


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "examples"


def run_source(source: str) -> Interpreter:
    tokens = Lexer(source, filename="<test>").lex()
    program = Parser(tokens, filename="<test>").parse()
    interpreter = Interpreter(program, filename="<test>")
    interpreter.run_main()
    return interpreter


class CoreLanguageTests(unittest.TestCase):
    def test_list_literal_and_indexing(self):
        interpreter = run_source(
            """
module sample
fn main() {
  values = [10, 20, 30]
  print(values[0])
  print(values[2])
}
"""
        )
        self.assertEqual(interpreter.output, ["10", "30"])

    def test_map_literal_and_indexing(self):
        interpreter = run_source(
            """
module sample
fn main() {
  scores = {"ada": 99, "grace": 95}
  print(scores["ada"])
}
"""
        )
        self.assertEqual(interpreter.output, ["99"])

    def test_for_loop_over_list(self):
        interpreter = run_source(
            """
module sample
fn main() {
  total = 0
  for value in [1, 2, 3, 4] {
    total = total + value
  }
  print(total)
}
"""
        )
        self.assertEqual(interpreter.output, ["10"])

    def test_for_loop_over_map_keys(self):
        interpreter = run_source(
            """
module sample
fn main() {
  scores = {"ada": 2, "grace": 3}
  total = 0
  for name in scores {
    total = total + scores[name]
  }
  print(total)
}
"""
        )
        self.assertEqual(interpreter.output, ["5"])

    def test_record_literal_and_field_access(self):
        interpreter = run_source(
            """
module sample
fn main() {
  user = User { name: "Ada", scores: [90, 95] }
  print(user.name)
  print(user.scores[1])
}
"""
        )
        self.assertEqual(interpreter.output, ["Ada", "95"])

    def test_boolean_condition_does_not_parse_as_record_literal(self):
        interpreter = run_source(
            """
module sample
fn main() {
  ready = true
  if ready {
    print("ok")
  }
}
"""
        )
        self.assertEqual(interpreter.output, ["ok"])

    def test_invalid_index_target_has_diagnostic(self):
        tokens = Lexer(
            """
module sample
fn main() {
  value = 1
  print(value[0])
}
""",
            filename="<test>",
        ).lex()
        program = Parser(tokens, filename="<test>").parse()
        with self.assertRaises(Exception) as context:
            Interpreter(program, filename="<test>").run_main()
        self.assertEqual(context.exception.diagnostic.code, "invalid_index_target")

    def test_unknown_record_field_has_diagnostic(self):
        tokens = Lexer(
            """
module sample
fn main() {
  user = User { name: "Ada" }
  print(user.email)
}
""",
            filename="<test>",
        ).lex()
        program = Parser(tokens, filename="<test>").parse()
        with self.assertRaises(Exception) as context:
            Interpreter(program, filename="<test>").run_main()
        self.assertEqual(context.exception.diagnostic.code, "unknown_field")

    def test_for_loop_over_non_collection_has_diagnostic(self):
        tokens = Lexer(
            """
module sample
fn main() {
  for value in 42 {
    print(value)
  }
}
""",
            filename="<test>",
        ).lex()
        program = Parser(tokens, filename="<test>").parse()
        with self.assertRaises(Exception) as context:
            Interpreter(program, filename="<test>").run_main()
        self.assertEqual(context.exception.diagnostic.code, "not_iterable")

    def test_len_on_non_collection_has_diagnostic(self):
        tokens = Lexer(
            """
module sample
fn main() {
  print(len(42))
}
""",
            filename="<test>",
        ).lex()
        program = Parser(tokens, filename="<test>").parse()
        with self.assertRaises(Exception) as context:
            Interpreter(program, filename="<test>").run_main()
        self.assertEqual(context.exception.diagnostic.code, "invalid_len_target")


class CoreLanguageCliTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_data_core_example_runs(self):
        result = self.run_cli("run", str(EXAMPLES / "data_core.hayulo"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), ["Ada", "language", "285"])

    def test_data_core_example_tests_pass(self):
        result = self.run_cli("test", str(EXAMPLES / "data_core.hayulo"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("4 passed, 0 failed", result.stdout)


if __name__ == "__main__":
    unittest.main()
