from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.checker import HayuloStaticError, check_program
from hayulo.lexer import Lexer
from hayulo.parser import Parser


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "examples"


def parse_source(source: str):
    tokens = Lexer(source, filename="<test>").lex()
    return Parser(tokens, filename="<test>").parse()


def assert_static_code(testcase: unittest.TestCase, source: str, code: str) -> None:
    with testcase.assertRaises(HayuloStaticError) as context:
        check_program(parse_source(source), filename="<test>")
    testcase.assertEqual(context.exception.diagnostic.code, code)


class StaticCheckerTests(unittest.TestCase):
    def test_valid_examples_pass_static_check(self):
        for path in [EXAMPLES / "hello.hayulo", EXAMPLES / "data_core.hayulo"]:
            program = Parser(Lexer(path.read_text(encoding="utf-8"), filename=str(path)).lex(), filename=str(path)).parse()
            check_program(program, filename=str(path))

    def test_unknown_name(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  print(missing)
}
""",
            "name.unknown_symbol",
        )

    def test_wrong_function_arity(self):
        assert_static_code(
            self,
            """
module bad
fn add(a: Int, b: Int) -> Int {
  return a + b
}
fn main() {
  print(add(1))
}
""",
            "call.arity_mismatch",
        )

    def test_local_type_inference_feeds_return_check(self):
        assert_static_code(
            self,
            """
module bad
fn main() -> Int {
  let value = "not an int"
  return value
}
""",
            "type.return_mismatch",
        )

    def test_explicit_return_type_check(self):
        assert_static_code(
            self,
            """
module bad
fn title() -> Text {
  return 42
}
""",
            "type.return_mismatch",
        )

    def test_argument_type_check(self):
        assert_static_code(
            self,
            """
module bad
fn double(value: Int) -> Int {
  return value + value
}
fn main() {
  print(double("x"))
}
""",
            "type.argument_mismatch",
        )

    def test_record_field_check(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  let user = User { name: "Ada" }
  print(user.email)
}
""",
            "record.unknown_field",
        )

    def test_record_field_check_uses_later_function_return_inference(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  let user = profile()
  print(user.email)
}
fn profile() {
  return User { name: "Ada" }
}
""",
            "record.unknown_field",
        )

    def test_for_loop_iterable_check(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  for value in 42 {
    print(value)
  }
}
""",
            "type.not_iterable",
        )

    def test_reassignment_before_binding(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  set value = 1
}
""",
            "name.reassignment_before_binding",
        )

    def test_duplicate_binding(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  let value = 1
  let value = 2
}
""",
            "name.duplicate_definition",
        )

    def test_try_requires_option_or_result(self):
        assert_static_code(
            self,
            """
module bad
fn main() {
  let value = try 1
}
""",
            "type.invalid_try_target",
        )

    def test_try_checks_early_return_compatibility(self):
        assert_static_code(
            self,
            """
module bad
fn maybe() -> Option<Int> {
  return None
}
fn main() -> Int {
  let value = try maybe()
  return value
}
""",
            "type.try_return_mismatch",
        )

    def test_match_requires_exhaustive_option_cases(self):
        assert_static_code(
            self,
            """
module bad
fn maybe() -> Option<Int> {
  return Some(1)
}
fn main() {
  let value = maybe()
  match value {
    Some(item) => {
      print(item)
    }
  }
}
""",
            "match.non_exhaustive",
        )


class StaticCheckerCliTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src"), "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_check_json_reports_static_diagnostic(self):
        with tempfile.NamedTemporaryFile("w", suffix=".hayulo", delete=False) as tmp:
            tmp.write(
                """
module bad
fn add(a: Int, b: Int) -> Int {
  return a + b
}
fn main() {
  print(add(1))
}
"""
            )
            path = tmp.name
        try:
            result = self.run_cli("check", path, "--json")
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stderr, "")
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "failed")
            self.assertEqual(payload["errors"][0]["code"], "call.arity_mismatch")
            self.assertEqual(payload["errors"][0]["details"]["expected"], 2)
            self.assertEqual(payload["errors"][0]["details"]["actual"], 1)
        finally:
            Path(path).unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
