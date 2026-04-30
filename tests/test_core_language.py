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
  let values = [10, 20, 30]
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
  let scores = {"ada": 99, "grace": 95}
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
  let total = 0
  for value in [1, 2, 3, 4] {
    set total = total + value
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
  let scores = {"ada": 2, "grace": 3}
  let total = 0
  for name in scores {
    set total = total + scores[name]
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
  let user = User { name: "Ada", scores: [90, 95] }
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
  let ready = true
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
  let value = 1
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
  let user = User { name: "Ada" }
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

    def test_option_result_try_and_match(self):
        interpreter = run_source(
            """
module sample
fn find_user(id: Int) -> Option<User> {
  if id == 1 {
    return Some(User { name: "Ada" })
  }
  return None
}

fn user_name(id: Int) -> Result<Text, Text> {
  let user = try find_user(id)
  return Ok(user.name)
}

fn main() {
  let result = user_name(1)
  match result {
    Ok(name) => {
      print(name)
    }
    Err(error) => {
      print(error)
    }
  }
}
"""
        )
        self.assertEqual(interpreter.output, ["Ada"])

    def test_old_bare_assignment_has_migration_diagnostic(self):
        with self.assertRaises(Exception) as context:
            Parser(
                Lexer(
                    """
module sample
fn main() {
  value = 1
}
""",
                    filename="<test>",
                ).lex(),
                filename="<test>",
            ).parse()
        self.assertEqual(context.exception.diagnostic.code, "syntax.binding_requires_let_or_set")

    def test_postfix_try_has_migration_diagnostic(self):
        with self.assertRaises(Exception) as context:
            Lexer(
                """
module sample
fn maybe() -> Option<Int> {
  return Some(1)
}
fn main() {
  let value = maybe()?
}
""",
                filename="<test>",
            ).lex()
        self.assertEqual(context.exception.diagnostic.code, "syntax.postfix_try_removed")


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
