from lib import runner
from lib.runner import COMMANDS, _command_kwargs, run_all, run_many
from test_support import create_sample_repo


def test_runner_registry_matches_expected_commands():
    assert set(COMMANDS) == {
        "scan",
        "measure",
        "config",
        "git",
        "graph",
        "symbols",
        "tests",
    }


def test_run_all_returns_all_command_results(tmp_path):
    repo = create_sample_repo(tmp_path)
    result = run_all(str(repo))

    assert set(result) == set(COMMANDS)
    assert result["scan"]["summary"]["total_files"] >= 4


def test_runner_supports_lang_and_multi_repo(tmp_path):
    repo_one = create_sample_repo(tmp_path / "one")
    repo_two = create_sample_repo(tmp_path / "two")

    assert _command_kwargs("scan", "python") == {"lang_filter": "python"}
    assert _command_kwargs("config", "python") == {}

    result = run_many([str(repo_one), str(repo_two)], "python")
    assert set(result) == {str(repo_one), str(repo_two)}
    assert "python" in result[str(repo_one)]["measure"]


def test_runner_captures_command_exceptions(monkeypatch):
    original = runner.COMMANDS["scan"]["fn"]

    monkeypatch.setitem(
        runner.COMMANDS["scan"],
        "fn",
        lambda repo, **kwargs: (_ for _ in ()).throw(RuntimeError("boom")),
    )

    result = run_all("repo")
    assert result["scan"] == {"error": "boom"}
    monkeypatch.setitem(runner.COMMANDS["scan"], "fn", original)
