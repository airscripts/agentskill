from commands import git as git_command
from commands.git import (
    _analyze_subjects,
    _detect_merge_strategy,
    _parse_commit_subject,
    analyze,
)
from test_support import create_sample_repo, git, init_git_repo, make_commit


def test_git_reports_empty_repository(tmp_path):
    repo = create_sample_repo(tmp_path)
    init_git_repo(repo)
    result = analyze(str(repo))

    assert result["error"] == "git log failed"
    assert result["script"] == "git"


def test_parse_commit_subject_and_subject_analysis():
    assert _parse_commit_subject("feat(api)!: add endpoint") == ("feat", "api", True)
    assert _parse_commit_subject("plain message") == (None, None, False)

    prefix_data, scope_counts, scoped_count, total, lengths, signed_count = (
        _analyze_subjects(
            "a" * 40
            + "|feat(api): add endpoint|test@example.com|G\n"
            + "b" * 40
            + "|fix: patch bug|test@example.com|N\n"
            + "c" * 40
            + "|plain message|test@example.com|G\n"
        )
    )

    assert total == 3
    assert scoped_count == 1
    assert scope_counts == {"api": 1}
    assert prefix_data["feat"]["count"] == 1
    assert prefix_data["unprefixed"]["count"] == 1
    assert signed_count == 2
    assert lengths[0] == len("feat(api): add endpoint")


def test_detect_merge_strategy_paths(monkeypatch):
    monkeypatch.setattr(git_command, "_run", lambda cmd, cwd: (1, ""))
    assert _detect_merge_strategy("repo") == ("unknown", "insufficient data")

    monkeypatch.setattr(git_command, "_run", lambda cmd, cwd: (0, ""))
    assert _detect_merge_strategy("repo") == ("rebase", "no merge commits in history")

    monkeypatch.setattr(git_command, "_run", lambda cmd, cwd: (0, "a\nb\n"))

    assert _detect_merge_strategy("repo") == (
        "squash",
        "merge commits have single parent",
    )

    monkeypatch.setattr(git_command, "_run", lambda cmd, cwd: (0, "a b\nc d\n"))

    assert _detect_merge_strategy("repo") == (
        "merge",
        "merge commits have multiple parents",
    )


def test_git_analyze_reports_real_history_and_branches(tmp_path):
    repo = create_sample_repo(tmp_path)
    init_git_repo(repo)

    make_commit(
        repo, "notes.txt", "hello\n", "feat(api): add endpoint", "Adds API endpoint"
    )

    make_commit(repo, "fix.txt", "hello\n", "fix: patch cli")
    git(repo, "branch", "feature/demo")
    git(repo, "branch", "bugfix/issue")

    result = analyze(str(repo))

    assert result["commits"]["total"] == 2
    assert result["commits"]["prefixes"]["feat"]["count"] == 1
    assert result["commits"]["scoped"]["uses_scopes"] is True
    assert result["commits"]["has_body"]["pct_with_body"] == 50.0
    assert result["branches"]["prefixes"] == {"bugfix/": 1, "feature/": 1}
    assert result["merge_strategy"]["detected"] == "rebase"


def test_git_analyze_can_report_empty_history(monkeypatch, tmp_path):
    repo = create_sample_repo(tmp_path)
    init_git_repo(repo)

    def fake_run(cmd, cwd):
        if "--format=%H|%s|%ae|%G?" in cmd:
            return 0, ""
        return 0, ""

    monkeypatch.setattr(git_command, "_run", fake_run)
    result = analyze(str(repo))

    assert result == {"error": "empty repository", "script": "git"}
