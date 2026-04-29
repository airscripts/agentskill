"""Interactive gap detection and prompt handling for AGENTS generation."""

from dataclasses import dataclass
from typing import Protocol

from lib.agents_document import AgentsSection, build_section
from lib.references import ReferenceDocument

_REFERENCE_TEST_COMMAND_PATTERNS = [
    r"Run command:\s*`([^`]+)`",
    r"canonical test command:\s*`([^`]+)`",
]

_REFERENCE_COMMIT_PREFIX_PATTERNS = [
    r"Commit prefixes observed:\s*`([^`]+)`",
    r"Preferred commit prefixes:\s*`([^`]+)`",
]

_REFERENCE_MERGE_STRATEGY_PATTERNS = [
    r"Merge strategy:\s*`([^`]+)`",
    r"Preferred merge strategy:\s*`([^`]+)`",
]


class PromptIO(Protocol):
    def ask(self, prompt: str) -> str: ...


class StdinPromptIO:
    def ask(self, prompt: str) -> str:
        return input(prompt)


@dataclass(frozen=True)
class GenerationGap:
    key: str
    section: str
    prompt: str
    inferred_value: str | None = None


def _first_run_command(analysis: dict) -> str | None:
    tests = analysis.get("tests", {})

    if not isinstance(tests, dict):
        return None

    for lang_data in tests.values():
        if not isinstance(lang_data, dict):
            continue

        run_command = lang_data.get("run_command")

        if isinstance(run_command, str) and run_command and run_command != "unknown":
            return run_command

    return None


def _search_reference_patterns(
    documents: list[ReferenceDocument],
    patterns: list[str],
) -> str | None:
    import re

    values: list[str] = []

    for document in documents:
        for pattern in patterns:
            match = re.search(pattern, document.content, re.IGNORECASE)

            if match is None:
                continue

            value = match.group(1).strip()

            if value and value not in values:
                values.append(value)

    if len(values) == 1:
        return values[0]

    return None


def detect_generation_gaps(
    analysis: dict,
    reference_documents: list[ReferenceDocument] | None = None,
) -> list[GenerationGap]:
    documents = reference_documents or []
    gaps: list[GenerationGap] = []

    if _first_run_command(analysis) is None:
        gaps.append(
            GenerationGap(
                key="test_command",
                section="testing",
                prompt=(
                    "I couldn't determine the canonical test command. "
                    "Enter it, or press Enter to skip: "
                ),
                inferred_value=_search_reference_patterns(
                    documents, _REFERENCE_TEST_COMMAND_PATTERNS
                ),
            )
        )

    git = analysis.get("git", {})

    if isinstance(git, dict) and "error" in git:
        gaps.append(
            GenerationGap(
                key="commit_prefixes",
                section="git",
                prompt=(
                    "Git history is unavailable. Enter preferred commit prefixes "
                    "(for example: feat:, fix:, chore:), or press Enter to skip: "
                ),
                inferred_value=_search_reference_patterns(
                    documents, _REFERENCE_COMMIT_PREFIX_PATTERNS
                ),
            )
        )

        gaps.append(
            GenerationGap(
                key="merge_strategy",
                section="git",
                prompt=(
                    "Git history is unavailable. Enter the preferred merge strategy "
                    "(for example: rebase, squash, merge), or press Enter to skip: "
                ),
                inferred_value=_search_reference_patterns(
                    documents, _REFERENCE_MERGE_STRATEGY_PATTERNS
                ),
            )
        )

    return gaps


def ask_generation_questions(
    gaps: list[GenerationGap],
    prompt_io: PromptIO,
) -> dict[str, str]:
    answers: dict[str, str] = {}

    for gap in gaps:
        if gap.inferred_value is not None:
            answers[gap.key] = gap.inferred_value
            continue

        answer = prompt_io.ask(gap.prompt).strip()

        if answer:
            answers[gap.key] = answer

    return answers


def interactive_section_notes(answers: dict[str, str]) -> dict[str, list[str]]:
    notes: dict[str, list[str]] = {}

    test_command = answers.get("test_command")

    if test_command:
        for section in ("commands and workflows", "testing"):
            notes.setdefault(section, []).append(
                f"Use `{test_command}` as the canonical test command."
            )

    commit_prefixes = answers.get("commit_prefixes")

    if commit_prefixes:
        notes.setdefault("git", []).append(
            f"Preferred commit prefixes: `{commit_prefixes}`."
        )

    merge_strategy = answers.get("merge_strategy")

    if merge_strategy:
        notes.setdefault("git", []).append(
            f"Preferred merge strategy: `{merge_strategy}`."
        )

    return notes


def apply_interactive_notes(
    sections: dict[str, AgentsSection],
    notes: dict[str, list[str]],
) -> dict[str, AgentsSection]:
    if not notes:
        return sections

    updated = dict(sections)

    for section_name, entries in notes.items():
        section = updated.get(section_name)

        if section is None or not entries:
            continue

        prefix = "Interactive answers:\n" + "\n".join(f"- {entry}" for entry in entries)
        body = prefix + "\n\n" + section.body

        updated[section_name] = build_section(
            section.heading_text,
            body,
            heading_level=section.heading_level,
        )

    return updated
