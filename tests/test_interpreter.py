from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from hayulo.interpreter import Interpreter
from hayulo.lexer import Lexer
from hayulo.parser import Parser


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "examples"


def parse_source(source: str):
    tokens = Lexer(source, filename="<test>").lex()
    return Parser(tokens, filename="<test>").parse()


class InterpreterTests(unittest.TestCase):
    def test_greet(self):
        source = '''
module sample

fn greet(name: Text) -> Text {
  return "Hello, " + name
}

fn main() {
  print(greet("Ada"))
}
'''
        program = parse_source(source)
        interpreter = Interpreter(program, filename="<test>")
        interpreter.run_main()
        self.assertEqual(interpreter.output, ["Hello, Ada"])

    def test_conditionals(self):
        source = '''
module sample

fn grade(score: Int) -> Text {
  if score >= 90 {
    return "A"
  } else if score >= 80 {
    return "B"
  } else {
    return "C"
  }
}

test "grade works" {
  expect grade(91) == "A"
  expect grade(82) == "B"
  expect grade(50) == "C"
}
'''
        program = parse_source(source)
        results = Interpreter(program, filename="<test>").run_tests()
        self.assertTrue(all(result.passed for result in results))

    def test_expect_failure(self):
        source = '''
module sample

test "fails" {
  expect 1 == 2
}
'''
        program = parse_source(source)
        results = Interpreter(program, filename="<test>").run_tests()
        self.assertEqual(len(results), 1)
        self.assertFalse(results[0].passed)
        self.assertEqual(results[0].error, "Expectation failed.")


class CliTests(unittest.TestCase):
    def run_cli(self, *args: str):
        return subprocess.run(
            [sys.executable, "-m", "hayulo", *args],
            cwd=ROOT,
            env={"PYTHONPATH": str(ROOT / "src")},
            text=True,
            capture_output=True,
            check=False,
        )

    def test_check_json(self):
        result = self.run_cli("check", str(EXAMPLES / "hello.hayulo"), "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "ok")
        self.assertIn("main", payload["functions"])

    def test_run_hello(self):
        result = self.run_cli("run", str(EXAMPLES / "hello.hayulo"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Hello, human", result.stdout)

    def test_syntax_error_json(self):
        with tempfile.NamedTemporaryFile("w", suffix=".hayulo", delete=False) as tmp:
            tmp.write('module broken\nfn main( {\n')
            path = tmp.name
        try:
            result = self.run_cli("check", path, "--json")
            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "failed")
            self.assertIn("errors", payload)
        finally:
            Path(path).unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
