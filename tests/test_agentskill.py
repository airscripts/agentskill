"""Unit tests for agentskill."""

import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from agentskill.constants import (
    CASE_CAMEL, CASE_KEBAB, CASE_MIXED, CASE_PASCAL,
    CASE_SCREAMING_SNAKE, CASE_SNAKE,
    NAME_VAR, NAME_FUNCTION, NAME_TYPE, NAME_CONST,
)
from agentskill.analyzers.language.rust import RustAnalyzer
from agentskill.analyzers.language.python import PythonAnalyzer
from agentskill.synthesis import AgentSynthesizer, SynthesisConfig


class TestRunner:
    def __init__(self):
        self.passed = 0
        self.failed = 0

    def run(self, test_class):
        print(f"\n{test_class.__doc__ or test_class.__name__}")
        print("-" * 40)
        instance = test_class()
        for name in sorted(dir(test_class)):
            if name.startswith("test_"):
                method = getattr(instance, name)
                try:
                    if hasattr(method, '__code__') and 'tmp_path' in method.__code__.co_varnames:
                        with tempfile.TemporaryDirectory() as td:
                            method(Path(td))
                    else:
                        method()
                    print(f"  PASS  {name}")
                    self.passed += 1
                except AssertionError as e:
                    print(f"  FAIL  {name}: {e}")
                    self.failed += 1
                except Exception as e:
                    print(f"  ERROR {name}: {e}")
                    self.failed += 1

    def summary(self):
        total = self.passed + self.failed
        print(f"\n{'=' * 40}")
        print(f"Results: {self.passed}/{total} passed")
        return 1 if self.failed else 0


class TestRustAnalyzer:
    """Tests for RustAnalyzer."""

    def test_case_detection(self):
        analyzer = RustAnalyzer()
        assert analyzer.detect_case_style("MAX_SIZE") == CASE_SCREAMING_SNAKE
        assert analyzer.detect_case_style("max_size") == CASE_SNAKE
        assert analyzer.detect_case_style("max-size") == CASE_KEBAB
        assert analyzer.detect_case_style("maxSize") == CASE_CAMEL
        assert analyzer.detect_case_style("MaxSize") == CASE_PASCAL

    def test_analyze_rust_file(self, tmp_path):
        rs_file = tmp_path / "test.rs"
        rs_file.write_text(
            "fn main() {\n"
            "    let x = 1;\n"
            "    let my_var = get_value().unwrap();\n"
            "}\n"
            "struct MyStruct { field: i32 }\n"
            "const MAX_SIZE: usize = 100;\n"
        )

        analyzer = RustAnalyzer()
        result = analyzer.analyze_files([rs_file])

        assert result.file_count == 1
        assert "unwrap" in result.error_handling
        assert result.error_handling["unwrap"] >= 1


class TestPythonAnalyzer:
    """Tests for PythonAnalyzer."""

    def test_case_detection(self):
        analyzer = PythonAnalyzer()
        assert analyzer.detect_case_style("my_variable") == CASE_SNAKE
        assert analyzer.detect_case_style("MyClass") == CASE_PASCAL
        assert analyzer.detect_case_style("MAX_SIZE") == CASE_SCREAMING_SNAKE

    def test_analyze_python_file(self, tmp_path):
        py_file = tmp_path / "test.py"
        py_file.write_text(
            "def my_function():\n"
            "    my_var = 1\n"
            "    # a comment\n"
            "    return my_var\n"
            "\n"
            "class MyClass:\n"
            "    pass\n"
        )

        analyzer = PythonAnalyzer()
        result = analyzer.analyze_files([py_file])

        assert result.file_count == 1
        assert "my_function" not in str(result.naming_patterns)  # just check it ran


class TestSynthesis:
    """Tests for AGENTS.md synthesis."""

    def test_basic_synthesis(self):
        synthesizer = AgentSynthesizer()
        analyses = [
            {
                "languages": {
                    "python": {
                        "naming": {"vars": {"dominant_case": "snake_case"}},
                        "file_count": 5,
                    }
                },
                "git": {
                    "commits": {"count": 10, "avg_length": 50, "common_prefixes": {"feat": 5}},
                    "branches": {"count": 3, "common_prefixes": {"feature": 2}},
                },
                "tooling": {"git": True},
            }
        ]

        output = synthesizer.synthesize(analyses, ["/tmp/test"])
        assert "AGENTS.md" in output or "Overview" in output
        assert "python" in output.lower()

    def test_synthesis_config(self):
        config = SynthesisConfig(include_git=False, include_tooling=False)
        synthesizer = AgentSynthesizer(config)

        analyses = [
            {
                "languages": {"python": {"file_count": 1}},
                "git": {"commits": {}},
                "tooling": {},
            }
        ]

        output = synthesizer.synthesize(analyses, ["/tmp/test"])
        assert "Git" not in output
        assert "Tooling" not in output


if __name__ == "__main__":
    runner = TestRunner()
    runner.run(TestRustAnalyzer)
    runner.run(TestPythonAnalyzer)
    runner.run(TestSynthesis)
    sys.exit(runner.summary())
