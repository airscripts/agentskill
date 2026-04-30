from test_support import create_sample_repo, write

from agentskill.lib.update_feedback import (
    FEEDBACK_FILENAME,
    SectionFeedback,
    UpdateFeedback,
    load_feedback,
    validate_feedback,
)


def test_load_feedback_returns_empty_when_file_is_missing(tmp_path):
    repo = create_sample_repo(tmp_path)
    assert load_feedback(repo) == UpdateFeedback()


def test_validate_feedback_normalizes_sections_and_preserve_names():
    result = validate_feedback(
        {
            "sections": {
                " Overview ": {
                    "prepend_notes": ["Mention deployments."],
                    "pinned_facts": ["Use pytest as the canonical test runner."],
                }
            },
            "preserve_sections": [" Red Lines ", "red lines"],
        }
    )

    assert result == UpdateFeedback(
        sections={
            "overview": SectionFeedback(
                prepend_notes=["Mention deployments."],
                pinned_facts=["Use pytest as the canonical test runner."],
            )
        },
        preserve_sections=["red lines"],
    )


def test_load_feedback_reads_repo_local_sidecar_file(tmp_path):
    repo = create_sample_repo(tmp_path)
    write(
        repo,
        FEEDBACK_FILENAME,
        (
            "{\n"
            '  "sections": {\n'
            '    "testing": {\n'
            '      "pinned_facts": ["Use pytest as the canonical test runner."]\n'
            "    }\n"
            "  }\n"
            "}\n"
        ),
    )

    feedback = load_feedback(repo)
    assert feedback.sections["testing"].pinned_facts == [
        "Use pytest as the canonical test runner."
    ]


def test_validate_feedback_rejects_non_object_root():
    try:
        validate_feedback([])
        raise AssertionError("should have raised ValueError")
    except ValueError as exc:
        assert str(exc) == "feedback must be an object"


def test_validate_feedback_rejects_unknown_section_keys():
    try:
        validate_feedback(
            {
                "sections": {
                    "overview": {
                        "unknown": ["nope"],
                    }
                }
            }
        )
        raise AssertionError("should have raised ValueError")
    except ValueError as exc:
        assert str(exc) == "unsupported feedback keys for section overview: unknown"


def test_validate_feedback_rejects_invalid_preserve_sections_shape():
    try:
        validate_feedback({"preserve_sections": "testing"})
        raise AssertionError("should have raised ValueError")
    except ValueError as exc:
        assert str(exc) == "feedback.preserve_sections must be a list of strings"


def test_validate_feedback_rejects_non_string_list_items():
    try:
        validate_feedback(
            {
                "sections": {
                    "testing": {
                        "pinned_facts": ["pytest", 1],
                    }
                }
            }
        )
        raise AssertionError("should have raised ValueError")
    except ValueError as exc:
        assert str(exc) == (
            "feedback.sections.testing.pinned_facts must be a list of strings"
        )
