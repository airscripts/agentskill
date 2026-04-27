"""Gap detection and targeted question generation from reference adaptation results."""

from dataclasses import dataclass

from lib.reference_adaptation import AdaptedConvention, ReferenceAdaptationResult
from lib.references import ReferenceSource

QUESTION_CATEGORY_LANGUAGE = "language"
QUESTION_CATEGORY_FORMATTER = "formatter"
QUESTION_CATEGORY_LINTER = "linter"
QUESTION_CATEGORY_TYPE_CHECKER = "type_checker"
QUESTION_CATEGORY_TESTING = "testing"
QUESTION_CATEGORY_DIRECTORY_STRUCTURE = "directory_structure"
QUESTION_CATEGORY_CONFLICT = "conflict"
QUESTION_CATEGORY_UNKNOWN = "unknown"


_KNOWN_TOOLS: dict[str, set[str]] = {
    "testing": {
        "pytest",
        "unittest",
        "jest",
        "vitest",
        "go test",
        "cargo test",
        "rspec",
    },
    "formatter": {"ruff", "black", "prettier", "gofmt", "rustfmt"},
    "linter": {"ruff", "eslint", "golangci-lint", "clippy"},
    "type_checker": {"mypy", "pyright", "typescript"},
}


_ECOSYSTEM_MAP: dict[str, set[str]] = {
    "python": {"pytest", "unittest", "ruff", "black", "mypy", "pyright"},
    "typescript": {"jest", "vitest", "eslint", "prettier", "typescript"},
    "javascript": {"jest", "vitest", "eslint", "prettier"},
    "go": {"go test", "gofmt", "golangci-lint"},
    "rust": {"cargo test", "rustfmt", "clippy"},
    "ruby": {"rspec"},
}


_LANGUAGE_KEYWORDS = {
    "python": ["python", ".py", "pyproject.toml"],
    "typescript": ["typescript", ".ts", "tsconfig"],
    "javascript": ["javascript", ".js"],
    "go": ["go", ".go", "go.mod"],
    "rust": ["rust", ".rs", "cargo"],
}


def _extract_known_tools(text: str) -> set[str]:
    found: set[str] = set()
    text_lower = text.lower()

    for _cat, tools in _KNOWN_TOOLS.items():
        for tool in tools:
            if tool in text_lower:
                found.add(tool)

    return found


def _extract_section_languages(text: str) -> set[str]:
    found: set[str] = set()
    text_lower = text.lower()

    for lang, keywords in _LANGUAGE_KEYWORDS.items():
        for kw in keywords:
            if kw.lower() in text_lower:
                found.add(lang)
                break

    return found


def _target_languages(target_analysis: dict) -> set[str]:
    found: set[str] = set()
    summary = target_analysis.get("scan", {}).get("summary", {})

    for lang in summary.get("languages", []):
        lang_lower = lang.lower()
        for known in _LANGUAGE_KEYWORDS:
            if known in lang_lower or lang_lower in known:
                found.add(known)

    return found


def _same_ecosystem(lang: str, tool: str) -> bool:
    return tool in _ECOSYSTEM_MAP.get(lang, set())


@dataclass(frozen=True)
class ReferenceQuestion:
    section: str
    question: str
    reason: str
    category: str
    source: ReferenceSource | None = None
    blocking: bool = False
    options: list[str] | None = None

    def to_dict(self) -> dict:
        d: dict = {
            "section": self.section,
            "question": self.question,
            "reason": self.reason,
            "category": self.category,
            "blocking": self.blocking,
        }

        if self.source is not None:
            d["source"] = self.source.to_dict()

        if self.options is not None:
            d["options"] = self.options

        return d


def _question_from_uncertain(
    conv: AdaptedConvention,
    source: ReferenceSource | None,
    target_analysis: dict | None,
) -> ReferenceQuestion | None:
    cat = conv.category
    text = (conv.section.heading + " " + conv.section.body).lower()
    tools = _extract_known_tools(text)

    if cat in ("testing",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["testing"]), None)

        if tool:
            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target test framework is unclear. Should generated instructions mention {tool}, another framework, or omit test guidance?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_TESTING,
                source=source,
                blocking=False,
                options=[tool, "another framework", "omit test guidance"],
            )

    if cat in ("formatter",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["formatter"]), None)

        if tool:
            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target config does not show {tool}. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_FORMATTER,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat in ("linter",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["linter"]), None)

        if tool:
            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target config does not show {tool}. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_LINTER,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat == "type_checker":
        tool = next((t for t in tools if t in _KNOWN_TOOLS["type_checker"]), None)

        if tool:
            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target config does not show {tool}. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_TYPE_CHECKER,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat == "directory_structure":
        return ReferenceQuestion(
            section=conv.section.heading,
            question="The reference repo mentions directory paths, but the target structure is unclear. Should generated instructions include these directories?",
            reason=conv.reason,
            category=QUESTION_CATEGORY_DIRECTORY_STRUCTURE,
            source=source,
            blocking=False,
            options=["include", "omit", "ask later"],
        )

    if cat == "language":
        return ReferenceQuestion(
            section=conv.section.heading,
            question="The reference mentions a language, but the target language analysis is missing. Should the language convention be applied?",
            reason=conv.reason,
            category=QUESTION_CATEGORY_LANGUAGE,
            source=source,
            blocking=False,
            options=["apply", "omit", "review manually"],
        )

    if cat == "unknown" and conv.section.body.strip():
        return ReferenceQuestion(
            section=conv.section.heading,
            question=f"The reference section '{conv.section.heading}' could not be matched to detected target conventions. Should it influence the generated AGENTS.md?",
            reason=conv.reason,
            category=QUESTION_CATEGORY_UNKNOWN,
            source=source,
            blocking=False,
            options=["use it", "ignore it", "review manually"],
        )

    return None


def _question_from_mismatch(
    conv: AdaptedConvention,
    source: ReferenceSource | None,
    target_analysis: dict | None,
) -> ReferenceQuestion | None:
    cat = conv.category
    text = (conv.section.heading + " " + conv.section.body).lower()
    tools = _extract_known_tools(text)

    if cat == "language":
        ref_langs = _extract_section_languages(text)

        if target_analysis:
            target_langs = _target_languages(target_analysis)

            if ref_langs and target_langs and not (ref_langs & target_langs):
                return None

    if cat in ("testing",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["testing"]), None)

        if tool:
            if target_analysis:
                target_langs = _target_languages(target_analysis)
                relevant = any(_same_ecosystem(lang, tool) for lang in target_langs)

                if not relevant:
                    return None

            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but a different test framework is detected in the target. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_TESTING,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat in ("formatter",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["formatter"]), None)

        if tool:
            if target_analysis:
                target_langs = _target_languages(target_analysis)
                relevant = any(_same_ecosystem(lang, tool) for lang in target_langs)

                if not relevant:
                    return None

            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target config does not show {tool}. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_FORMATTER,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat in ("linter",):
        tool = next((t for t in tools if t in _KNOWN_TOOLS["linter"]), None)

        if tool:
            if target_analysis:
                target_langs = _target_languages(target_analysis)
                relevant = any(_same_ecosystem(lang, tool) for lang in target_langs)

                if not relevant:
                    return None

            return ReferenceQuestion(
                section=conv.section.heading,
                question=f"The reference repo uses {tool}, but the target config does not show {tool}. Should this convention be applied?",
                reason=conv.reason,
                category=QUESTION_CATEGORY_LINTER,
                source=source,
                blocking=False,
                options=["apply", "omit", "use target-detected tooling only"],
            )

    if cat == "directory_structure":
        return ReferenceQuestion(
            section=conv.section.heading,
            question="The reference repo mentions directory paths not found in the target. Should generated instructions include these directories?",
            reason=conv.reason,
            category=QUESTION_CATEGORY_DIRECTORY_STRUCTURE,
            source=source,
            blocking=False,
            options=["include", "omit", "ask later"],
        )

    return None


def _detect_conflicts(
    adaptations: list[ReferenceAdaptationResult],
    target_analysis: dict | None,
) -> list[ReferenceQuestion]:
    by_category: dict[str, list[tuple[AdaptedConvention, ReferenceSource | None]]] = {}

    for result in adaptations:
        for conv in result.conventions:
            if conv.status not in ("uncertain", "mismatched"):
                continue

            if conv.category not in by_category:
                by_category[conv.category] = []

            by_category[conv.category].append((conv, result.source))

    questions: list[ReferenceQuestion] = []

    for cat, entries in by_category.items():
        if len(entries) < 2:
            continue

        all_tools: set[str] = set()

        for conv, _src in entries:
            text = (conv.section.heading + " " + conv.section.body).lower()
            all_tools |= _extract_known_tools(text)

        cat_tools: set[str] = set()

        tool_cat_map: dict[str, str] = {}

        for tool in all_tools:
            for tc, tools in _KNOWN_TOOLS.items():
                if tool in tools:
                    cat_tools.add(tool)
                    tool_cat_map[tool] = tc
                    break

        by_tool_cat: dict[str, set[str]] = {}

        for tool in cat_tools:
            tc = tool_cat_map[tool]

            if tc not in by_tool_cat:
                by_tool_cat[tc] = set()

            by_tool_cat[tc].add(tool)

        for tc, conflicting in by_tool_cat.items():
            if len(conflicting) < 2:
                continue

            sorted_tools = sorted(conflicting)
            tool_list = ", ".join(sorted_tools)

            qcat = QUESTION_CATEGORY_TESTING
            if tc == "formatter":
                qcat = QUESTION_CATEGORY_FORMATTER
            elif tc == "linter":
                qcat = QUESTION_CATEGORY_LINTER
            elif tc == "type_checker":
                qcat = QUESTION_CATEGORY_TYPE_CHECKER

            options = list(sorted_tools) + ["omit", "use target-detected tooling only"]

            questions.append(
                ReferenceQuestion(
                    section=cat,
                    question=f"Multiple references suggest different {qcat} conventions: {tool_list}. Which should be used?",
                    reason=f"conflicting {qcat} tools across references",
                    category=QUESTION_CATEGORY_CONFLICT,
                    blocking=False,
                    options=options,
                )
            )

    return questions


def _dedup_key(q: ReferenceQuestion) -> tuple:
    return (q.category, q.section, q.question)


def generate_reference_questions(
    adaptations: list[ReferenceAdaptationResult],
    target_analysis: dict | None = None,
) -> list[ReferenceQuestion]:
    questions: list[ReferenceQuestion] = []
    seen: set[tuple] = set()

    for result in adaptations:
        for conv in result.conventions:
            q: ReferenceQuestion | None = None

            if conv.status == "uncertain":
                q = _question_from_uncertain(conv, result.source, target_analysis)
            elif conv.status == "mismatched":
                q = _question_from_mismatch(conv, result.source, target_analysis)

            if q is not None:
                key = _dedup_key(q)

                if key not in seen:
                    seen.add(key)
                    questions.append(q)

    conflicts = _detect_conflicts(adaptations, target_analysis)

    for q in conflicts:
        key = _dedup_key(q)

        if key not in seen:
            seen.add(key)
            questions.append(q)

    return questions
