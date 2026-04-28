"""Repo-local feedback loading for AGENTS.md update workflows."""

import json
from dataclasses import dataclass, field
from pathlib import Path

from lib.agents_document import normalize_section_name

FEEDBACK_FILENAME = ".agentskill-feedback.json"
SUPPORTED_SECTION_FEEDBACK_KEYS = {"prepend_notes", "pinned_facts"}


@dataclass(frozen=True)
class SectionFeedback:
    """Explicit feedback for one regenerated section."""

    prepend_notes: list[str] = field(default_factory=list)
    pinned_facts: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class UpdateFeedback:
    """Normalized update feedback loaded from the repo."""

    sections: dict[str, SectionFeedback] = field(default_factory=dict)
    preserve_sections: list[str] = field(default_factory=list)


def empty_feedback() -> UpdateFeedback:
    return UpdateFeedback()


def _require_object(value: object, label: str) -> dict:
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be an object")

    return value


def _validate_string_list(value: object, label: str) -> list[str]:
    if not isinstance(value, list):
        raise ValueError(f"{label} must be a list of strings")

    for item in value:
        if not isinstance(item, str):
            raise ValueError(f"{label} must be a list of strings")

    return value


def _dedupe_preserving_order(values: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []

    for value in values:
        if value in seen:
            continue

        seen.add(value)
        ordered.append(value)

    return ordered


def validate_feedback(data: object) -> UpdateFeedback:
    """Validate and normalize feedback data."""
    root = _require_object(data, "feedback")
    sections_data = root.get("sections", {})
    preserve_data = root.get("preserve_sections", [])

    if "sections" in root:
        sections_data = _require_object(sections_data, "feedback.sections")

    preserve_sections = (
        _validate_string_list(preserve_data, "feedback.preserve_sections")
        if "preserve_sections" in root
        else []
    )

    sections: dict[str, SectionFeedback] = {}

    for raw_name, raw_feedback in sections_data.items():
        if not isinstance(raw_name, str):
            raise ValueError("feedback.sections keys must be strings")

        normalized_name = normalize_section_name(raw_name)

        if normalized_name in sections:
            raise ValueError(
                f"duplicate feedback section after normalization: {raw_name}"
            )

        feedback_obj = _require_object(
            raw_feedback,
            f"feedback.sections.{raw_name}",
        )

        unknown_keys = sorted(
            key for key in feedback_obj if key not in SUPPORTED_SECTION_FEEDBACK_KEYS
        )

        if unknown_keys:
            raise ValueError(
                f"unsupported feedback keys for section {raw_name}: "
                + ", ".join(unknown_keys)
            )

        sections[normalized_name] = SectionFeedback(
            prepend_notes=_validate_string_list(
                feedback_obj.get("prepend_notes", []),
                f"feedback.sections.{raw_name}.prepend_notes",
            ),
            pinned_facts=_validate_string_list(
                feedback_obj.get("pinned_facts", []),
                f"feedback.sections.{raw_name}.pinned_facts",
            ),
        )

    return UpdateFeedback(
        sections=sections,
        preserve_sections=_dedupe_preserving_order(
            [normalize_section_name(name) for name in preserve_sections]
        ),
    )


def load_feedback(repo_path: str | Path) -> UpdateFeedback:
    """Load optional update feedback from a repository root."""
    root = Path(repo_path)
    feedback_path = root / FEEDBACK_FILENAME

    if not feedback_path.exists():
        return empty_feedback()

    try:
        raw = json.loads(feedback_path.read_text())
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid feedback JSON: {exc.msg}") from exc

    return validate_feedback(raw)
