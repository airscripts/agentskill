from test_support import create_repo, write

from agentskill.commands.measure import (
    _file_metrics,
    _measure_blank_lines_generic,
    _measure_blank_lines_python,
    _measure_indentation,
    _measure_line_lengths,
    measure,
)


def test_measure_indentation_variants():
    assert _measure_indentation(["\tline"]) == {"unit": "tabs", "size": 1}

    assert _measure_indentation(["    line", "        next"]) == {
        "unit": "spaces",
        "size": 4,
    }

    assert _measure_indentation(["  line", "    next"]) == {"unit": "spaces", "size": 2}
    assert _measure_indentation(["line"]) == {"unit": "unknown", "size": 0}


def test_measure_line_lengths_and_file_metrics(tmp_path):
    path = tmp_path / "sample.py"
    path.write_text("line  \nnext")
    metrics = _file_metrics(path)

    assert metrics["trailing_newline"] is False
    assert metrics["has_trailing_ws"] is True
    assert _measure_line_lengths([10, 20, 30, 40]) == {}
    assert _measure_line_lengths([10, 20, 30, 40, 50])["p95"] == 40


def test_measure_blank_lines_python_and_generic(tmp_path):
    py_file = tmp_path / "blank_lines.py"

    py_file.write_text(
        "import os\n\n\n"
        "class A:\n\n"
        "    def one(self):\n"
        "        return 1\n\n\n"
        "    def two(self):\n"
        "        return 2\n\n\n"
        "def three():\n"
        "    return 3\n"
    )

    ts_file = tmp_path / "blank_lines.ts"

    ts_file.write_text(
        "export function one() {\n"
        "  return 1\n"
        "}\n\n\n"
        "export function two() {\n"
        "  return 2\n"
        "}\n"
    )

    py_result = _measure_blank_lines_python([py_file])
    ts_result = _measure_blank_lines_generic([ts_file], "typescript")

    assert py_result["after_imports"]["mode"] == 2
    assert py_result["between_methods"]["mode"] == 2
    assert ts_result["between_top_level_defs"]["mode"] == 2


def test_measure_entrypoint_handles_lang_filters_and_missing_paths(tmp_path):
    repo = create_repo(tmp_path)
    write(repo, "pkg/main.py", "def run():\n    return 1\n")
    write(repo, "web/app.ts", "export function run() {\n  return 1\n}\n")

    result = measure(str(repo), "typescript")

    assert set(result) == {"typescript"}
    assert measure(str(repo / "missing"))["script"] == "measure"


def test_measure_reports_file_paths_as_invalid_repos(tmp_path):
    file_path = tmp_path / "sample.py"
    file_path.write_text("print('hi')\n")

    assert measure(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "measure",
    }


def test_measure_handles_empty_file_metrics_and_empty_repo(tmp_path):
    empty_file = tmp_path / "empty.py"
    empty_file.write_text("")

    metrics = _file_metrics(empty_file)

    assert metrics["indent"] == {"unit": "unknown", "size": 0}
    assert metrics["line_lengths"] == []
    assert metrics["trailing_newline"] is False
    assert metrics["has_trailing_ws"] is False

    repo = create_repo(tmp_path / "empty_repo", name="empty_repo")

    assert measure(str(repo)) == {}


def test_measure_repo_with_only_empty_files_has_stable_aggregate(tmp_path):
    repo = create_repo(tmp_path)
    write(repo, "pkg/a.py", "")
    write(repo, "pkg/b.py", "")

    result = measure(str(repo))

    assert result["python"]["indentation"] == {
        "unit": "spaces",
        "size": 4,
        "tab_files": [],
        "mixed_files": [],
    }

    assert result["python"]["line_length"] == {}
    assert result["python"]["trailing_newline"] == {"present": 0, "absent": 2}
    assert result["python"]["trailing_whitespace"] == {"files_with_trailing_ws": 0}


def test_measure_reports_mixed_tabs_spaces_and_trailing_newlines(tmp_path):
    repo = create_repo(tmp_path)
    write(repo, "pkg/tabs.py", "\tdef one():\n\t\treturn 1\n")
    write(repo, "pkg/spaces.py", "    def two():\n        return 2\n")
    write(repo, "pkg/mixed.py", "\tdef three():\n    return 3")

    result = measure(str(repo))
    indentation = result["python"]["indentation"]

    assert indentation["unit"] == "spaces"
    assert indentation["size"] == 4
    assert any(path.endswith("pkg/tabs.py") for path in indentation["tab_files"])
    assert any(path.endswith("pkg/mixed.py") for path in indentation["mixed_files"])
    assert result["python"]["trailing_newline"] == {"present": 2, "absent": 1}


def test_measure_handles_non_python_repo_and_blank_only_line_lengths(tmp_path):
    repo = create_repo(tmp_path)
    write(
        repo,
        "web/app.ts",
        "export function one() {\n  return 1\n}\n\n\nexport function two() {\n  return 2\n}\n",
    )

    write(repo, "web/blank.ts", "\n \n\t\n")
    result = measure(str(repo))

    assert set(result) == {"typescript"}
    assert result["typescript"]["line_length"]["max"] >= 1
    assert result["typescript"]["blank_lines"]["between_top_level_defs"]["mode"] == 2


def test_measure_line_lengths_stay_empty_for_blank_only_content():
    assert _measure_line_lengths([]) == {}
    assert _measure_line_lengths([0, 0, 0, 0]) == {}
