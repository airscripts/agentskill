from lib.interactive_runner import (
    GenerationGap,
    apply_interactive_notes,
    ask_generation_questions,
    detect_generation_gaps,
    interactive_section_notes,
)
from lib.references import ReferenceDocument, ReferenceSource
from lib.update_runner import render_agents_sections
from test_support import create_repo


class FakePromptIO:
    def __init__(self, answers: list[str]) -> None:
        self.answers = list(answers)
        self.prompts: list[str] = []

    def ask(self, prompt: str) -> str:
        self.prompts.append(prompt)

        if not self.answers:
            raise AssertionError("unexpected prompt")

        return self.answers.pop(0)


def _reference_document(content: str) -> ReferenceDocument:
    return ReferenceDocument(
        source=ReferenceSource(kind="local", value="../reference"),
        content=content,
    )


def test_detect_generation_gaps_for_missing_git_and_test_signals():
    analysis = {
        "tests": {},
        "git": {"error": "not a git repository", "script": "git"},
    }

    gaps = detect_generation_gaps(analysis)

    assert [gap.key for gap in gaps] == [
        "test_command",
        "commit_prefixes",
        "merge_strategy",
    ]


def test_reference_metadata_can_resolve_interactive_gaps():
    analysis = {
        "tests": {},
        "git": {"error": "not a git repository", "script": "git"},
    }

    reference = _reference_document(
        "# AGENTS\n\n"
        "## 12. Testing\n"
        "- Run command: `pytest`\n\n"
        "## 13. Git\n"
        "- Commit prefixes observed: `feat:, fix:`.\n"
        "- Merge strategy: `rebase`.\n"
    )

    gaps = detect_generation_gaps(analysis, [reference])

    assert [gap.inferred_value for gap in gaps] == [
        "pytest",
        "feat:, fix:",
        "rebase",
    ]


def test_ask_generation_questions_skips_blank_answers():
    gaps = [
        GenerationGap(
            key="test_command",
            section="testing",
            prompt="test prompt",
        )
    ]

    prompt_io = FakePromptIO([""])
    answers = ask_generation_questions(gaps, prompt_io)

    assert answers == {}
    assert prompt_io.prompts == ["test prompt"]


def test_apply_interactive_notes_prefixes_matching_sections(tmp_path):
    repo = create_repo(tmp_path, {"pkg/main.py": "def main():\n    return 1\n"})
    analysis = {
        "scan": {"summary": {"by_language": {"python": {}}, "languages": ["python"]}},
        "graph": {"monorepo_boundaries": {}},
        "measure": {},
        "symbols": {"python": {}},
        "tests": {"python": {"framework": "pytest", "run_command": "pytest"}},
        "config": {},
        "git": {"error": "not a git repository", "script": "git"},
    }

    sections = render_agents_sections(repo, analysis)
    notes = interactive_section_notes(
        {
            "test_command": "pytest -q",
            "commit_prefixes": "feat:, fix:",
        }
    )

    updated = apply_interactive_notes(sections, notes)

    assert updated["testing"].body.startswith("Interactive answers:\n")
    assert "Use `pytest -q` as the canonical test command." in updated["testing"].body
    assert "Preferred commit prefixes: `feat:, fix:`." in updated["git"].body
