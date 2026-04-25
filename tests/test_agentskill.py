#!/usr/bin/env python3
"""Self-contained test runner for agentskill tests."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))

from agentskill import (
    CASE_CAMEL,
    CASE_KEBAB,
    CASE_MIXED,
    CASE_PASCAL,
    CASE_SCREAMING_SNAKE,
    CASE_SNAKE,
    LANG_RUST,
    LANG_PYTHON,
    NAME_VAR,
    NAME_FUNCTION,
    NAME_TYPE,
    NAME_CONST,
    GIT_DIR,
    EXTENSIONS,
    TOOL_FILES,
    SKIP_DIRS,
    PYTHON_VAR_KEYWORDS,
    RUST_COMMENT_STYLES,
    PYTHON_COMMENT_STYLE,
    RUST_ERROR_PATTERNS,
    RUST_ERROR_KEYS,
    detect_case_style,
    extract_commit_prefixes,
    extract_branch_prefixes,
    is_git_repo,
    should_skip_dir,
    is_hidden_path,
    track_blank_lines,
    detect_comment_style,
    extract_rust_name_lengths,
    extract_python_name_lengths,
    process_file_for_style,
    analyze_code_style,
    count_rust_error_pattern,
    extract_rust_naming,
    process_rust_file,
    analyze_rust_files,
    scan_source_files,
    detect_tooling,
    convert_dataclasses,
    validate_repos,
    output_report,
    analyze_repo,
)


class TestRunner:
    """Simple test runner that mimics pytest behavior."""

    def __init__(self):
        self.passed = 0
        self.failed = 0

    def run(self, test_class):
        """Run all test methods in a class."""
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
        """Print test summary."""
        total = self.passed + self.failed
        print(f"\n{'=' * 40}")
        print(f"Results: {self.passed}/{total} passed")
        if self.failed:
            print(f"         {self.failed}/{total} failed")
            return 1
        return 0


def make_git_repo(path: Path, commits=None):
    """Create a minimal git repo with optional commits."""
    subprocess.run(["git", "init", str(path)], capture_output=True, timeout=10)
    subprocess.run(["git", "-C", str(path), "config", "user.email", "test@test.com"], capture_output=True, timeout=10)
    subprocess.run(["git", "-C", str(path), "config", "user.name", "Test"], capture_output=True, timeout=10)

    if commits:
        for msg in commits:
            (path / "file.txt").write_text(msg)
            subprocess.run(["git", "-C", str(path), "add", "."], capture_output=True, timeout=10)
            subprocess.run(["git", "-C", str(path), "commit", "-m", msg], capture_output=True, timeout=10)


def make_source_tree(path: Path, files: dict):
    """Create a directory tree with source files. files = {rel_path: content}."""
    for rel, content in files.items():
        fpath = path / rel
        fpath.parent.mkdir(parents=True, exist_ok=True)
        fpath.write_text(content)


class TestDetectCaseStyle:
    """Tests for detect_case_style function."""

    def test_screaming_snake_case(self):
        assert detect_case_style("MAX_SIZE") == CASE_SCREAMING_SNAKE
        assert detect_case_style("API_KEY") == CASE_SCREAMING_SNAKE

    def test_snake_case(self):
        assert detect_case_style("max_size") == CASE_SNAKE
        assert detect_case_style("api_key") == CASE_SNAKE

    def test_kebab_case(self):
        assert detect_case_style("max-size") == CASE_KEBAB
        assert detect_case_style("api-key") == CASE_KEBAB

    def test_camel_case(self):
        assert detect_case_style("maxSize") == CASE_CAMEL
        assert detect_case_style("apiKey") == CASE_CAMEL

    def test_pascal_case(self):
        assert detect_case_style("MaxSize") == CASE_PASCAL
        assert detect_case_style("ApiKey") == CASE_PASCAL

    def test_mixed(self):
        assert detect_case_style("Max-Size") == CASE_MIXED
        assert detect_case_style("max_Size") == CASE_MIXED


class TestExtractCommitPrefixes:
    """Tests for extract_commit_prefixes function."""

    def test_no_prefixes(self):
        commits = ["fix bug", "update readme", "[feat] missing colon"]
        assert extract_commit_prefixes(commits) == {}

    def test_single_prefix(self):
        commits = ["[feat]: add new feature", "[feat]: update logic"]
        result = extract_commit_prefixes(commits)
        assert result == {"[feat]": 2}

    def test_multiple_prefixes(self):
        commits = [
            "[feat]: add new feature",
            "[fix]: resolve bug",
            "[feat]: update logic",
            "[fix]: another fix",
            "[docs]: update readme",
        ]
        result = extract_commit_prefixes(commits)
        assert result["[feat]"] == 2
        assert result["[fix]"] == 2
        assert result["[docs]"] == 1

    def test_empty_commits(self):
        assert extract_commit_prefixes([]) == {}


class TestExtractBranchPrefixes:
    """Tests for extract_branch_prefixes function."""

    def test_single_prefix(self):
        branches = ["feature/add-thing", "feature/update-thing"]
        result = extract_branch_prefixes(branches)
        assert result == {"feature": 2}

    def test_multiple_prefixes(self):
        branches = [
            "feature/add-thing",
            "fix/bug-1",
            "feature/update-thing",
            "fix/bug-2",
        ]
        result = extract_branch_prefixes(branches)
        assert result["feature"] == 2
        assert result["fix"] == 2

    def test_remotes_filtered(self):
        branches = ["remotes/origin/feature/add-thing", "feature/update-thing"]
        result = extract_branch_prefixes(branches)
        assert result["feature"] == 2

    def test_no_prefix(self):
        branches = ["main", "master"]
        assert extract_branch_prefixes(branches) == {}


class TestIsGitRepo:
    """Tests for is_git_repo function."""

    def test_non_git_directory(self, tmp_path):
        assert not is_git_repo(str(tmp_path))

    def test_git_directory(self, tmp_path):
        (tmp_path / ".git").mkdir()
        assert is_git_repo(str(tmp_path))


class TestShouldSkipDir:
    """Tests for should_skip_dir function."""

    def test_skip_node_modules(self):
        path = Path("/project/node_modules/some-package")
        assert should_skip_dir(path)

    def test_skip_target(self):
        path = Path("/rust-project/target/debug")
        assert should_skip_dir(path)

    def test_skip_pycache(self):
        path = Path("/python-project/__pycache__")
        assert should_skip_dir(path)

    def test_no_skip(self):
        path = Path("/project/src")
        assert not should_skip_dir(path)


class TestIsHiddenPath:
    """Tests for is_hidden_path function."""

    def test_hidden_directory(self):
        path = Path("/home/user/.config/app")
        assert is_hidden_path(path)

    def test_not_hidden(self):
        path = Path("/home/user/projects/app")
        assert not is_hidden_path(path)

    def test_hidden_file(self):
        path = Path("/home/user/.bashrc")
        assert is_hidden_path(path)


class TestConstants:
    """Tests that constants are properly defined and distinct."""

    def test_case_constants_distinct(self):
        cases = [CASE_SCREAMING_SNAKE, CASE_SNAKE, CASE_KEBAB, CASE_CAMEL, CASE_PASCAL, CASE_MIXED]
        assert len(set(cases)) == len(cases)

    def test_name_constants(self):
        assert NAME_VAR == "vars"
        assert NAME_FUNCTION == "functions"
        assert NAME_TYPE == "types"
        assert NAME_CONST == "consts"

    def test_language_constants(self):
        assert LANG_RUST == "rust"
        assert LANG_PYTHON == "python"


class TestTrackBlankLines:
    """Tests for track_blank_lines function."""

    def test_blank_line(self):
        counts = []
        prev, streak = track_blank_lines("", False, 0, counts)
        assert streak == 1
        assert prev is False
        assert counts == []

    def test_code_after_blanks(self):
        counts = []
        prev, streak = track_blank_lines("", True, 2, counts)
        assert streak == 3
        assert counts == []

    def test_code_line_with_prior_blank(self):
        counts = []
        prev, streak = track_blank_lines("x = 1", True, 3, counts)
        assert streak == 0
        assert counts == [3]

    def test_code_line_no_prior_blank(self):
        counts = []
        prev, streak = track_blank_lines("x = 1", True, 0, counts)
        assert streak == 0
        assert counts == []


class TestDetectCommentStyle:
    """Tests for detect_comment_style function."""

    def test_doc_comment(self):
        assert detect_comment_style("/// doc comment") == "///"
        assert detect_comment_style("/** block doc */") == "///"

    def test_inner_doc(self):
        assert detect_comment_style("//! inner doc") == "//!"

    def test_line_comment(self):
        assert detect_comment_style("// regular") == "//"

    def test_block_comment(self):
        assert detect_comment_style("/* block */") == "/*"

    def test_python_comment(self):
        assert detect_comment_style("# python") == "#"

    def test_not_comment(self):
        assert detect_comment_style("x = 1") is None

    def test_code_not_comment(self):
        assert detect_comment_style("let x = 1;") is None


class TestExtractRustNameLengths:
    """Tests for extract_rust_name_lengths function."""

    def test_let_binding(self):
        result = extract_rust_name_lengths("    let count = 0;")
        assert NAME_VAR in result
        assert result[NAME_VAR] == 5

    def test_let_mut(self):
        result = extract_rust_name_lengths("    let mut buffer = vec![];")
        assert NAME_VAR in result
        assert result[NAME_VAR] == 6

    def test_fn(self):
        result = extract_rust_name_lengths("fn do_something() {")
        assert NAME_FUNCTION in result
        assert result[NAME_FUNCTION] == 12

    def test_struct(self):
        result = extract_rust_name_lengths("struct MyStruct {")
        assert NAME_TYPE in result
        assert result[NAME_TYPE] == 8

    def test_const(self):
        result = extract_rust_name_lengths("const MAX_RETRIES: usize = 3;")
        assert NAME_CONST in result
        assert result[NAME_CONST] == 11

    def test_no_match(self):
        result = extract_rust_name_lengths("println!(\"hello\");")
        assert result == {}


class TestExtractPythonNameLengths:
    """Tests for extract_python_name_lengths function."""

    def test_var_assignment(self):
        result = extract_python_name_lengths("count = 0")
        assert NAME_VAR in result
        assert result[NAME_VAR] == 5

    def test_def(self):
        result = extract_python_name_lengths("def process_data():")
        assert NAME_FUNCTION in result
        assert result[NAME_FUNCTION] == 12

    def test_class(self):
        result = extract_python_name_lengths("class DataLoader:")
        assert NAME_TYPE in result
        assert result[NAME_TYPE] == 10

    def test_skip_keywords(self):
        for kw in PYTHON_VAR_KEYWORDS:
            if kw in ('if', 'for', 'while'):
                result = extract_python_name_lengths(f"{kw} x:")
            else:
                result = extract_python_name_lengths(f"{kw}.x = 1")
            assert NAME_VAR not in result

    def test_skip_commented_assignment(self):
        result = extract_python_name_lengths("# count = 0")
        assert NAME_VAR not in result

    def test_no_match(self):
        result = extract_python_name_lengths("print('hello')")
        assert result == {}


class TestCountRustErrorPattern:
    """Tests for count_rust_error_pattern function."""

    def test_unwrap(self):
        assert count_rust_error_pattern("x.unwrap()", "unwrap()")

    def test_expect(self):
        assert count_rust_error_pattern("x.expect(\"msg\")", "expect(")

    def test_question_mark(self):
        assert count_rust_error_pattern("x?;", "?")

    def test_question_mark_in_comment(self):
        assert not count_rust_error_pattern("// x?", "?")

    def test_panic(self):
        assert count_rust_error_pattern("panic!(\"err\");", "panic!")

    def test_result(self):
        assert count_rust_error_pattern("fn run() -> Result<bool, Error> {", "Result<")

    def test_no_match(self):
        assert not count_rust_error_pattern("let x = 1;", "unwrap()")


class TestExtractRustNaming:
    """Tests for extract_rust_naming function."""

    def test_var_naming(self):
        result = extract_rust_naming("let my_var = 1;")
        assert NAME_VAR in result
        name, style = result[NAME_VAR]
        assert name == "my_var"
        assert style == CASE_SNAKE

    def test_fn_naming(self):
        result = extract_rust_naming("fn do_thing() {")
        assert NAME_FUNCTION in result
        name, style = result[NAME_FUNCTION]
        assert name == "do_thing"
        assert style == CASE_SNAKE

    def test_struct_naming(self):
        result = extract_rust_naming("struct MyStruct {")
        assert NAME_TYPE in result
        name, style = result[NAME_TYPE]
        assert name == "MyStruct"
        assert style == CASE_PASCAL

    def test_const_naming(self):
        result = extract_rust_naming("const MAX_SIZE: usize = 10;")
        assert NAME_CONST in result
        name, style = result[NAME_CONST]
        assert name == "MAX_SIZE"
        assert style == CASE_SCREAMING_SNAKE


class TestAnalyzeCodeStyle:
    """Tests for analyze_code_style with temp files."""

    def test_python_style(self, tmp_path):
        py_file = tmp_path / "test.py"
        py_file.write_text("def my_function():\n    my_var = 1\n    # a comment\n\nclass MyClass:\n    pass\n")

        style = analyze_code_style([py_file], LANG_PYTHON)
        assert NAME_FUNCTION in style.naming_descriptiveness
        assert NAME_VAR in style.naming_descriptiveness
        assert NAME_TYPE in style.naming_descriptiveness

    def test_rust_style(self, tmp_path):
        rs_file = tmp_path / "test.rs"
        rs_file.write_text("fn my_function() {\n    let my_var = 1;\n}\n\nstruct MyStruct {\n    field: i32,\n}\n\nconst MAX_SIZE: usize = 10;\n")

        style = analyze_code_style([rs_file], LANG_RUST)
        assert NAME_FUNCTION in style.naming_descriptiveness
        assert NAME_VAR in style.naming_descriptiveness


class TestAnalyzeRustFiles:
    """Tests for analyze_rust_files with temp files."""

    def test_rust_analysis(self, tmp_path):
        rs_file = tmp_path / "test.rs"
        rs_file.write_text(
            "fn main() {\n"
            "    let x = get_value().unwrap();\n"
            "    let y = get_other()?;\n"
            "    panic!(\"error\");\n"
            "}\n"
            "struct MyStruct { field: i32 }\n"
            "const MAX_SIZE: usize = 10;\n"
            "// a comment\n"
        )

        result = analyze_rust_files([rs_file])
        assert "naming" in result
        assert "error_handling" in result
        assert "comments" in result
        assert result["error_handling"]["unwrap"] >= 1
        assert result["error_handling"]["?"] >= 1
        assert result["error_handling"]["panic"] >= 1
        assert result["comments"]["comment_lines"] >= 1


class TestScanSourceFiles:
    """Tests for scan_source_files with temp directory."""

    def test_find_python(self, tmp_path):
        make_source_tree(tmp_path, {
            "src/main.py": "x = 1",
            "src/utils.py": "y = 2",
        })
        result = scan_source_files(str(tmp_path))
        assert "python" in result
        assert len(result["python"]) == 2

    def test_find_rust(self, tmp_path):
        make_source_tree(tmp_path, {
            "src/main.rs": "fn main() {}",
        })
        result = scan_source_files(str(tmp_path))
        assert "rust" in result
        assert len(result["rust"]) == 1

    def test_skip_hidden(self, tmp_path):
        make_source_tree(tmp_path, {
            ".hidden/secret.py": "x = 1",
            "src/visible.py": "y = 2",
        })
        result = scan_source_files(str(tmp_path))
        assert "python" in result
        assert len(result["python"]) == 1

    def test_skip_node_modules(self, tmp_path):
        make_source_tree(tmp_path, {
            "node_modules/pkg/index.js": "x = 1",
            "src/app.js": "y = 2",
        })
        result = scan_source_files(str(tmp_path))
        assert "javascript" in result
        assert len(result["javascript"]) == 1

    def test_mixed_languages(self, tmp_path):
        make_source_tree(tmp_path, {
            "src/main.py": "x = 1",
            "src/main.rs": "fn main() {}",
            "src/index.js": "y = 2",
            "src/main.go": "func main() {}",
        })
        result = scan_source_files(str(tmp_path))
        assert "python" in result
        assert "rust" in result
        assert "javascript" in result
        assert "go" in result


class TestDetectTooling:
    """Tests for detect_tooling with temp directories."""

    def test_cargo_toml(self, tmp_path):
        (tmp_path / "Cargo.toml").write_text("[package]\nname = \"test\"")
        result = detect_tooling(str(tmp_path))
        assert result.get("cargo")

    def test_github_actions(self, tmp_path):
        (tmp_path / ".github" / "workflows").mkdir(parents=True)
        (tmp_path / ".github" / "workflows" / "ci.yml").write_text("on: push")
        result = detect_tooling(str(tmp_path))
        assert result.get("GitHub Actions CI")

    def test_makefile(self, tmp_path):
        (tmp_path / "Makefile").write_text("all:\n\techo hi")
        result = detect_tooling(str(tmp_path))
        assert result.get("make")

    def test_empty_dir(self, tmp_path):
        result = detect_tooling(str(tmp_path))
        assert result == {}


class TestConvertDataclasses:
    """Tests for convert_dataclasses function."""

    def test_plain_dict(self):
        data = {"key": "value", "num": 42}
        assert convert_dataclasses(data) == data

    def test_nested_dict(self):
        data = {"a": {"b": 1, "c": [1, 2]}}
        assert convert_dataclasses(data) == data

    def test_list(self):
        data = [{"a": 1}, {"b": 2}]
        assert convert_dataclasses(data) == data

    def test_dataclass_conversion(self):
        from agentskill import NamingPatterns
        obj = NamingPatterns()
        result = convert_dataclasses(obj)
        assert isinstance(result, dict)
        assert NAME_VAR in result


class TestValidateRepos:
    """Tests for validate_repos function."""

    def test_valid_directory(self, tmp_path):
        result = validate_repos([str(tmp_path)])
        assert result == [str(tmp_path)]

    def test_invalid_directory(self):
        result = validate_repos(["/nonexistent/path/xyz"])
        assert result == []

    def test_mixed(self, tmp_path):
        result = validate_repos([str(tmp_path), "/nonexistent/path/xyz"])
        assert result == [str(tmp_path)]


class TestOutputReport:
    """Tests for output_report function."""

    def test_stdout(self, capsys=None):
        report = {"repos": ["/tmp/test"], "analyses": []}
        output_report(report)
        assert True

    def test_file_output(self, tmp_path):
        report = {"repos": ["/tmp/test"], "analyses": []}
        outfile = tmp_path / "report.json"
        output_report(report, str(outfile))
        assert outfile.exists()
        data = json.loads(outfile.read_text())
        assert "repos" in data


class TestAnalyzeRepo:
    """Integration tests for analyze_repo with real git repos."""

    def test_analyze_git_repo(self, tmp_path):
        make_git_repo(tmp_path, commits=[
            "[feat]: add initial code",
            "[fix]: fix bug",
        ])
        make_source_tree(tmp_path, {
            "src/main.py": "def my_function():\n    my_var = 1\n\nclass MyClass:\n    pass\n",
            "src/utils.py": "def helper():\n    x = 42\n",
        })

        result = analyze_repo(str(tmp_path))
        assert "path" in result
        assert "git" in result
        assert "tooling" in result
        assert "languages" in result
        assert "python" in result["languages"]
        assert result["languages"]["python"]["file_count"] == 2

    def test_analyze_rust_repo(self, tmp_path):
        make_git_repo(tmp_path, commits=["[feat]: initial commit"])
        make_source_tree(tmp_path, {
            "src/main.rs": "fn main() {\n    let x = 1;\n    let y = get_val().unwrap();\n}\n\nstruct MyStruct { field: i32 }\n",
            "Cargo.toml": "[package]\nname = \"test\"\n",
        })

        result = analyze_repo(str(tmp_path))
        assert "rust" in result["languages"]
        assert result["tooling"].get("cargo")

    def test_non_git_directory(self, tmp_path):
        make_source_tree(tmp_path, {
            "main.py": "x = 1",
        })
        result = analyze_repo(str(tmp_path))
        assert "path" in result


if __name__ == "__main__":
    runner = TestRunner()

    runner.run(TestDetectCaseStyle)
    runner.run(TestExtractCommitPrefixes)
    runner.run(TestExtractBranchPrefixes)
    runner.run(TestIsGitRepo)
    runner.run(TestShouldSkipDir)
    runner.run(TestIsHiddenPath)
    runner.run(TestConstants)
    runner.run(TestTrackBlankLines)
    runner.run(TestDetectCommentStyle)
    runner.run(TestExtractRustNameLengths)
    runner.run(TestExtractPythonNameLengths)
    runner.run(TestCountRustErrorPattern)
    runner.run(TestExtractRustNaming)
    runner.run(TestAnalyzeCodeStyle)
    runner.run(TestAnalyzeRustFiles)
    runner.run(TestScanSourceFiles)
    runner.run(TestDetectTooling)
    runner.run(TestConvertDataclasses)
    runner.run(TestValidateRepos)
    runner.run(TestOutputReport)
    runner.run(TestAnalyzeRepo)

    sys.exit(runner.summary())