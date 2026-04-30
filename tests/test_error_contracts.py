from agentskill.commands.config import detect
from agentskill.commands.git import analyze as analyze_git
from agentskill.commands.graph import build_graph
from agentskill.commands.measure import measure
from agentskill.commands.scan import scan
from agentskill.commands.symbols import extract_symbols
from agentskill.commands.tests import analyze_tests


def test_analyzers_return_exact_error_payload_for_invalid_file_paths(tmp_path):
    file_path = tmp_path / "not_a_repo.txt"
    file_path.write_text("hello\n")

    assert scan(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "scan",
    }

    assert measure(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "measure",
    }

    assert detect(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "config",
    }

    assert analyze_git(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "git",
    }

    assert build_graph(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "graph",
    }

    assert extract_symbols(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "symbols",
    }

    assert analyze_tests(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "tests",
    }
