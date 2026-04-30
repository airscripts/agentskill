import json
import sys
from pathlib import Path

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib

from test_support import create_repo, create_sample_repo, write

from agentskill.main import main as packaged_main


def test_cli_scan_outputs_json(tmp_path, capsys):
    repo = create_sample_repo(tmp_path)
    exit_code = packaged_main(["scan", str(repo), "--pretty"])

    assert exit_code == 0

    assert exit_code == 0

    output = json.loads(capsys.readouterr().out)
    assert output["summary"]["total_files"] >= 4


def test_cli_analyze_runs_all_commands(tmp_path, capsys):
    repo = create_sample_repo(tmp_path)
    exit_code = packaged_main(["analyze", str(repo), "--pretty"])

    assert exit_code == 0

    output = json.loads(capsys.readouterr().out)

    assert set(output) == {
        "scan",
        "measure",
        "config",
        "git",
        "graph",
        "symbols",
        "tests",
    }


def test_cli_analyze_accepts_reference_without_changing_output_shape(tmp_path, capsys):
    repo = create_sample_repo(tmp_path / "target")
    reference = create_repo(tmp_path, name="reference")
    write(reference, "AGENTS.md", "# AGENTS\n\n## 12. Testing\nUse pytest.\n")

    exit_code = packaged_main(
        ["analyze", str(repo), "--reference", str(reference), "--pretty"]
    )

    assert exit_code == 0
    output = json.loads(capsys.readouterr().out)

    assert set(output) == {
        "scan",
        "measure",
        "config",
        "git",
        "graph",
        "symbols",
        "tests",
    }


def test_cli_analyze_reports_invalid_reference_path(tmp_path, capsys):
    repo = create_sample_repo(tmp_path / "target")
    missing = tmp_path / "missing-reference"
    exit_code = packaged_main(["analyze", str(repo), "--reference", str(missing)])

    assert exit_code == 1
    assert capsys.readouterr().err == f"reference path does not exist: {missing}\n"


def test_cli_writes_out_file_and_multi_repo_results(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    repo_one = create_sample_repo(tmp_path / "one")
    repo_two = create_sample_repo(tmp_path / "two")
    out_file = Path("report.json")

    exit_code = packaged_main(
        ["analyze", str(repo_one), str(repo_two), "--out", str(out_file)]
    )

    assert exit_code == 0

    payload = json.loads(out_file.read_text())
    assert set(payload) == {str(repo_one), str(repo_two)}


def test_pyproject_includes_cli_module_for_console_script():
    with Path("pyproject.toml").open("rb") as f:
        data = tomllib.load(f)

    assert data["project"]["scripts"]["agentskill"] == "agentskill.main:main"
    assert "py-modules" not in data.get("tool", {}).get("setuptools", {})
    assert data["tool"]["setuptools"]["packages"]["find"]["include"] == ["agentskill*"]
