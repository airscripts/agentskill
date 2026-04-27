"""Tests for reference question generation."""

from lib.reference_adaptation import (
    AdaptedConvention,
    ReferenceAdaptationResult,
    ReferenceSection,
)
from lib.reference_questions import (
    QUESTION_CATEGORY_CONFLICT,
    QUESTION_CATEGORY_DIRECTORY_STRUCTURE,
    QUESTION_CATEGORY_FORMATTER,
    QUESTION_CATEGORY_LINTER,
    QUESTION_CATEGORY_TESTING,
    QUESTION_CATEGORY_UNKNOWN,
    ReferenceQuestion,
    generate_reference_questions,
)
from lib.references import ReferenceSource


def _src(kind: str = "local", value: str = "../ref") -> ReferenceSource:
    return ReferenceSource(kind=kind, value=value)


def _section(
    heading: str = "Testing", body: str = "Use pytest.", level: int = 2
) -> ReferenceSection:
    return ReferenceSection(heading=heading, body=body, level=level)


def _conv(
    heading: str = "Testing",
    body: str = "Use pytest.",
    category: str = "testing",
    status: str = "uncertain",
    reason: str = "missing config",
) -> AdaptedConvention:
    return AdaptedConvention(
        section=_section(heading, body),
        category=category,
        status=status,
        reason=reason,
    )


def _result(
    conventions: list[AdaptedConvention] | None = None,
    source: ReferenceSource | None = None,
) -> ReferenceAdaptationResult:
    if conventions is None:
        conventions = [_conv()]
    if source is None:
        source = _src()

    return ReferenceAdaptationResult(source=source, conventions=conventions)


def _analysis(**kwargs) -> dict:
    return dict(kwargs)


def test_question_serialization_required_fields():
    q = ReferenceQuestion(
        section="Testing",
        question="Use pytest?",
        reason="missing config",
        category="testing",
    )
    d = q.to_dict()

    assert d["section"] == "Testing"
    assert d["question"] == "Use pytest?"
    assert d["reason"] == "missing config"
    assert d["category"] == "testing"
    assert d["blocking"] is False
    assert "source" not in d
    assert "options" not in d


def test_question_serialization_optional_fields():
    q = ReferenceQuestion(
        section="Testing",
        question="Use pytest?",
        reason="missing config",
        category="testing",
        source=_src(),
        blocking=True,
        options=["pytest", "omit"],
    )
    d = q.to_dict()

    assert d["source"] == {"kind": "local", "value": "../ref"}
    assert d["blocking"] is True
    assert d["options"] == ["pytest", "omit"]


def test_uncertain_testing_question():
    conv = _conv(
        heading="Testing",
        body="Use pytest for tests.",
        category="testing",
        status="uncertain",
        reason="target analysis missing config data",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    q = questions[0]

    assert q.category == QUESTION_CATEGORY_TESTING
    assert "pytest" in q.question
    assert q.blocking is False
    assert q.options is not None
    assert "pytest" in q.options


def test_uncertain_formatter_question():
    conv = _conv(
        heading="Formatting",
        body="Use black for formatting.",
        category="formatter",
        status="uncertain",
        reason="target analysis missing config data",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_FORMATTER
    assert "black" in questions[0].question


def test_uncertain_linter_question():
    conv = _conv(
        heading="Linting",
        body="Use ruff for linting.",
        category="linter",
        status="uncertain",
        reason="target analysis missing config data",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_LINTER
    assert "ruff" in questions[0].question


def test_uncertain_directory_structure_question():
    conv = _conv(
        heading="Structure",
        body="Source code lives in src/.",
        category="directory_structure",
        status="uncertain",
        reason="no directory paths referenced",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_DIRECTORY_STRUCTURE


def test_uncertain_unknown_section_question():
    conv = _conv(
        heading="Philosophy",
        body="Be pragmatic.",
        category="unknown",
        status="uncertain",
        reason="no recognizable language, tool, or directory keywords",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_UNKNOWN
    assert "Philosophy" in questions[0].question


def test_mismatched_relevant_tool_question():
    conv = _conv(
        heading="Formatting",
        body="Use black for formatting.",
        category="formatter",
        status="mismatched",
        reason="tool black mentioned but not detected in target",
    )
    target = _analysis(
        scan={"summary": {"languages": ["python"]}},
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result], target_analysis=target)

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_FORMATTER
    assert "black" in questions[0].question


def test_mismatched_irrelevant_language_no_question():
    conv = _conv(
        heading="Go",
        body="Use gofmt for Go formatting.",
        category="language",
        status="mismatched",
        reason="language go not found in target scan summary",
    )
    target = _analysis(
        scan={"summary": {"languages": ["python"]}},
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result], target_analysis=target)

    assert len(questions) == 0


def test_mismatched_irrelevant_tool_no_question():
    conv = _conv(
        heading="Formatting",
        body="Use gofmt for Go formatting.",
        category="formatter",
        status="mismatched",
        reason="tool gofmt mentioned but not detected in target",
    )
    target = _analysis(
        scan={"summary": {"languages": ["python"]}},
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result], target_analysis=target)

    assert len(questions) == 0


def test_mismatched_directory_structure_question():
    conv = _conv(
        heading="Structure",
        body="Source code lives in frontend/.",
        category="directory_structure",
        status="mismatched",
        reason="referenced paths not found in target scan tree",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 1
    assert questions[0].category == QUESTION_CATEGORY_DIRECTORY_STRUCTURE


def test_applicable_convention_no_question():
    conv = _conv(
        heading="Testing",
        body="Use pytest.",
        category="testing",
        status="applicable",
        reason="pytest detected in target analysis",
    )
    result = _result(conventions=[conv])
    questions = generate_reference_questions([result])

    assert len(questions) == 0


def test_conflict_question():
    conv_a = _conv(
        heading="Testing",
        body="Use pytest.",
        category="testing",
        status="uncertain",
        reason="missing config",
    )
    conv_b = _conv(
        heading="Testing",
        body="Use unittest.",
        category="testing",
        status="uncertain",
        reason="missing config",
    )
    src_a = _src(value="ref-a")
    src_b = _src(value="ref-b")
    result_a = _result(conventions=[conv_a], source=src_a)
    result_b = _result(conventions=[conv_b], source=src_b)
    questions = generate_reference_questions([result_a, result_b])

    conflict = [q for q in questions if q.category == QUESTION_CATEGORY_CONFLICT]

    assert len(conflict) == 1
    assert "pytest" in conflict[0].question
    assert "unittest" in conflict[0].question
    assert conflict[0].options is not None
    assert "pytest" in conflict[0].options
    assert "unittest" in conflict[0].options


def test_deduplication():
    conv = _conv(
        heading="Testing",
        body="Use pytest.",
        category="testing",
        status="uncertain",
        reason="missing config",
    )
    result_a = _result(conventions=[conv], source=_src(value="ref-a"))
    result_b = _result(conventions=[conv], source=_src(value="ref-b"))
    questions = generate_reference_questions([result_a, result_b])

    testing = [q for q in questions if q.category == QUESTION_CATEGORY_TESTING]

    assert len(testing) == 1


def test_deterministic_order():
    conv_a = _conv(
        heading="Testing",
        body="Use pytest.",
        category="testing",
        status="uncertain",
        reason="missing config",
    )
    conv_b = _conv(
        heading="Formatting",
        body="Use black.",
        category="formatter",
        status="uncertain",
        reason="missing config",
    )
    result = _result(conventions=[conv_a, conv_b])
    questions_a = generate_reference_questions([result])
    questions_b = generate_reference_questions([result])

    assert [q.section for q in questions_a] == [q.section for q in questions_b]


def test_no_adaptations_returns_empty():
    questions = generate_reference_questions([])

    assert questions == []
