from commands.measure import (
    _file_metrics,
    _measure_blank_lines_generic,
    _measure_blank_lines_python,
    _measure_indentation,
    _measure_line_lengths,
    measure,
)
from test_support import create_repo, write


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
