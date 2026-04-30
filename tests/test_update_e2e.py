from test_support import create_sample_repo, write

from agentskill.main import main


def test_update_succeeds_without_feedback_file(tmp_path):
    repo = create_sample_repo(tmp_path)
    exit_code = main(["update", str(repo)])

    assert exit_code == 0
    assert (repo / "AGENTS.md").exists()


def test_feedback_biases_targeted_regeneration(tmp_path):
    repo = create_sample_repo(tmp_path)
    write(
        repo,
        ".agentskill-feedback.json",
        (
            "{\n"
            '  "sections": {\n'
            '    "overview": {\n'
            '      "prepend_notes": ["Mention that deployments go through GitHub Actions."],\n'
            '      "pinned_facts": ["Use pytest as the canonical test runner."]\n'
            "    }\n"
            "  }\n"
            "}\n"
        ),
    )

    exit_code = main(["update", str(repo), "--section", "overview"])
    assert exit_code == 0

    agents_text = (repo / "AGENTS.md").read_text()
    assert "Mention that deployments go through GitHub Actions." in agents_text
    assert "Use pytest as the canonical test runner." in agents_text


def test_feedback_preserve_sections_prevents_normal_regeneration(tmp_path):
    repo = create_sample_repo(tmp_path)
    write(
        repo,
        ".agentskill-feedback.json",
        '{\n  "preserve_sections": ["testing"]\n}\n',
    )

    write(
        repo,
        "AGENTS.md",
        (
            "# AGENTS\n\n"
            "## 1. Overview\n"
            "Old overview.\n"
            "## 12. Testing\n"
            "Keep this testing guidance exactly.\n"
        ),
    )

    exit_code = main(["update", str(repo)])
    assert exit_code == 0

    agents_text = (repo / "AGENTS.md").read_text()
    assert "Keep this testing guidance exactly.\n" in agents_text
    assert "Old overview.\n" not in agents_text


def test_feedback_preserve_sections_are_ignored_in_force_mode(tmp_path):
    repo = create_sample_repo(tmp_path)
    write(
        repo,
        ".agentskill-feedback.json",
        '{\n  "preserve_sections": ["testing"]\n}\n',
    )

    write(
        repo,
        "AGENTS.md",
        ("# AGENTS\n\n## 12. Testing\nKeep this testing guidance exactly.\n"),
    )

    exit_code = main(["update", str(repo), "--force"])
    assert exit_code == 0

    agents_text = (repo / "AGENTS.md").read_text()
    assert "Keep this testing guidance exactly.\n" not in agents_text
    assert "## 12. Testing\n" in agents_text


def test_malformed_feedback_fails_clearly(tmp_path, capsys):
    repo = create_sample_repo(tmp_path)
    write(
        repo,
        ".agentskill-feedback.json",
        '{\n  "sections": {\n    "overview": {"unknown": ["bad"]}\n  }\n}\n',
    )

    exit_code = main(["update", str(repo)])

    assert exit_code == 1
    assert "unsupported feedback keys for section overview: unknown" in (
        capsys.readouterr().err
    )
