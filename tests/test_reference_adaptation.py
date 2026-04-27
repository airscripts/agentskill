"""Tests for reference adaptation engine."""

from lib.reference_adaptation import (
    adapt_reference,
    adapt_references,
    split_markdown_sections,
)
from lib.references import ReferenceDocument, ReferenceSource


def test_split_markdown_sections_splits_on_headings():
    content = "# Project\n\nIntro\n\n## Testing\n\nUse pytest.\n\n## Style\n\nUse ruff."
    sections = split_markdown_sections(content)

    assert len(sections) == 3
    assert sections[0].heading == "Project"
    assert sections[0].level == 1
    assert "Intro" in sections[0].body

    assert sections[1].heading == "Testing"
    assert sections[1].level == 2
    assert "pytest" in sections[1].body

    assert sections[2].heading == "Style"
    assert sections[2].level == 2
    assert "ruff" in sections[2].body


def test_split_markdown_sections_no_headings():
    content = "Just some plain text.\nNo headings here."
    sections = split_markdown_sections(content)

    assert len(sections) == 1
    assert sections[0].heading == ""
    assert sections[0].level == 0
    assert "Just some plain text" in sections[0].body


def _make_doc(content: str) -> ReferenceDocument:
    src = ReferenceSource(kind="local", value="../ref")
    return ReferenceDocument(source=src, content=content)


def _make_analysis(**kwargs) -> dict:
    return dict(kwargs)


def test_adapt_reference_applicable_language():
    doc = _make_doc("## Python conventions\n\nUse Python 3.11 for this project.")
    target = _make_analysis(
        scan={"summary": {"languages": ["python"]}},
    )

    result = adapt_reference(doc, target)

    assert len(result.conventions) == 1
    assert result.conventions[0].status == "applicable"
    assert result.conventions[0].category == "language"
    assert "python found" in result.conventions[0].reason


def test_adapt_reference_mismatched_language():
    doc = _make_doc("## Go conventions\n\nUse gofmt for formatting.")
    target = _make_analysis(
        scan={"summary": {"languages": ["typescript"]}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "mismatched"
    assert "go not found" in result.conventions[0].reason


def test_adapt_reference_uncertain_missing_analysis():
    doc = _make_doc("## Testing\n\nUse pytest for tests.")
    target = _make_analysis()

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "uncertain"
    assert (
        "missing" in result.conventions[0].reason.lower()
        or "uncertain" in result.conventions[0].reason.lower()
    )


def test_adapt_reference_applicable_tool():
    doc = _make_doc("## Linting\n\nUse ruff for linting.")
    target = _make_analysis(
        config={"python": {"linter": {"name": "ruff"}}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "applicable"
    assert "ruff" in result.conventions[0].reason


def test_adapt_reference_mismatched_tool():
    doc = _make_doc("## Linting\n\nUse eslint for linting.")
    target = _make_analysis(
        config={"python": {"linter": {"name": "ruff"}}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "mismatched"
    assert "eslint" in result.conventions[0].reason


def test_adapt_reference_applicable_testing_framework():
    doc = _make_doc("## Testing\n\nUse pytest for unit tests.")
    target = _make_analysis(
        tests={"frameworks": [{"name": "pytest"}]},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "applicable"
    assert "pytest" in result.conventions[0].reason


def test_adapt_reference_mismatched_testing():
    doc = _make_doc("## Testing\n\nUse jest for testing.")
    target = _make_analysis(
        tests={"frameworks": [{"name": "pytest"}]},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "mismatched"
    assert "jest" in result.conventions[0].reason


def test_adapt_reference_directory_structure_applicable():
    doc = _make_doc("## Structure\n\nSource code lives in src/ and tests in tests/.")
    target = _make_analysis(
        scan={"tree": [{"path": "src/main.py"}, {"path": "tests/test_main.py"}]},
    )

    result = adapt_reference(doc, target)

    assert any(c.status == "applicable" for c in result.conventions)


def test_adapt_reference_directory_structure_mismatched():
    doc = _make_doc("## Structure\n\nSource code lives in frontend/ and backend/.")
    target = _make_analysis(
        scan={"tree": [{"path": "src/main.py"}]},
    )

    result = adapt_reference(doc, target)

    assert any(c.status == "mismatched" for c in result.conventions)


def test_adapt_reference_unknown_section_uncertain():
    doc = _make_doc("## Philosophy\n\nBe pragmatic about abstractions.")
    target = _make_analysis(
        scan={"summary": {"languages": ["python"]}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "uncertain"
    assert result.conventions[0].category == "unknown"


def test_adapt_reference_properties():
    doc = _make_doc(
        "## Python\n\nUse Python.\n\n## Go\n\nUse Go.\n\n## Unknown\n\nSomething."
    )
    target = _make_analysis(scan={"summary": {"languages": ["python"]}})

    result = adapt_reference(doc, target)

    assert len(result.applicable) == 1
    assert len(result.mismatched) == 1
    assert len(result.uncertain) == 1
    assert len(result.ignored) == 0


def test_adapt_references_preserves_order():
    docs = [
        _make_doc("## Python\n\nUse Python."),
        _make_doc("## Go\n\nUse Go."),
        _make_doc("## Rust\n\nUse Rust."),
    ]
    target = _make_analysis(scan={"summary": {"languages": ["python"]}})

    results = adapt_references(docs, target)

    assert len(results) == 3
    assert results[0].conventions[0].status == "applicable"
    assert results[1].conventions[0].status == "mismatched"
    assert results[2].conventions[0].status == "mismatched"


def test_adapt_reference_tolerates_missing_keys():
    doc = _make_doc("## Python\n\nUse Python.")
    target = _make_analysis()

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "uncertain"


def test_adapt_reference_multiple_sections():
    doc = _make_doc(
        "# Overview\n\nGeneral rules.\n\n## Python\n\nUse pytest.\n\n## Style\n\nUse ruff."
    )
    target = _make_analysis(
        scan={"summary": {"languages": ["python"]}},
        config={"python": {"linter": {"name": "ruff"}}},
        tests={"frameworks": [{"name": "pytest"}]},
    )

    result = adapt_reference(doc, target)

    assert len(result.conventions) == 3
    statuses = {c.status for c in result.conventions}
    assert "applicable" in statuses


def test_adapt_reference_git_convention():
    doc = _make_doc("## Git\n\nUse conventional commits.")
    target = _make_analysis(
        git={"commit_patterns": {"fix": 10}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "applicable"
    assert result.conventions[0].category == "git"


def test_adapt_reference_formatter_convention():
    doc = _make_doc("## Formatting\n\nUse black for formatting.")
    target = _make_analysis(
        config={"python": {"formatter": {"name": "black"}}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "applicable"
    assert result.conventions[0].category == "formatter"


def test_adapt_reference_linter_uncertain_no_config():
    doc = _make_doc("## Linting\n\nUse ruff for linting.")
    target = _make_analysis(
        scan={"summary": {"languages": ["python"]}},
    )

    result = adapt_reference(doc, target)

    assert result.conventions[0].status == "uncertain"
    assert "missing config" in result.conventions[0].reason.lower()


def test_adapt_reference_result_source_is_document_source():
    src = ReferenceSource(kind="local", value="../ref")
    doc = ReferenceDocument(source=src, content="## Python\n\nUse Python.")
    target = _make_analysis(scan={"summary": {"languages": ["python"]}})

    result = adapt_reference(doc, target)

    assert result.source is src
