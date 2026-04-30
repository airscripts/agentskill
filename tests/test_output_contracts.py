import json

from contract_utils import assert_matches_contract
from test_support import EXAMPLES_DIR

from agentskill.commands.config import detect
from agentskill.commands.graph import build_graph
from agentskill.commands.scan import scan
from agentskill.commands.symbols import extract_symbols
from agentskill.lib.runner import run_all
from agentskill.main import main


def test_analyze_python_example_matches_cli_contract(capsys):
    repo = EXAMPLES_DIR / "python"
    exit_code = main(["analyze", str(repo), "--pretty"])

    assert exit_code == 0
    actual = json.loads(capsys.readouterr().out)
    assert_matches_contract(actual, "analyze_python.json")


def test_analyze_mixed_example_matches_contract():
    actual = run_all(str(EXAMPLES_DIR / "mixed"))
    assert_matches_contract(actual, "analyze_mixed.json")


def test_scan_python_example_matches_contract():
    actual = scan(str(EXAMPLES_DIR / "python"))
    assert_matches_contract(actual, "scan_python.json")


def test_config_mixed_example_matches_contract():
    actual = detect(str(EXAMPLES_DIR / "mixed"))
    assert_matches_contract(actual, "config_mixed.json")


def test_graph_mixed_example_matches_contract():
    actual = build_graph(str(EXAMPLES_DIR / "mixed"))
    assert_matches_contract(actual, "graph_mixed.json")


def test_symbols_python_example_matches_contract():
    actual = extract_symbols(str(EXAMPLES_DIR / "python"))
    assert_matches_contract(actual, "symbols_python.json")
