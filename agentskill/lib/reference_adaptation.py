"""Reference adaptation engine for comparing reference AGENTS.md against target analysis."""

from dataclasses import dataclass

from agentskill.lib.references import ReferenceDocument, ReferenceSource


@dataclass(frozen=True)
class ReferenceSection:
    heading: str
    body: str
    level: int


@dataclass(frozen=True)
class AdaptedConvention:
    section: ReferenceSection
    category: str
    status: str
    reason: str


@dataclass(frozen=True)
class ReferenceAdaptationResult:
    source: ReferenceSource
    conventions: list[AdaptedConvention]

    @property
    def applicable(self) -> list[AdaptedConvention]:
        return [c for c in self.conventions if c.status == "applicable"]

    @property
    def mismatched(self) -> list[AdaptedConvention]:
        return [c for c in self.conventions if c.status == "mismatched"]

    @property
    def uncertain(self) -> list[AdaptedConvention]:
        return [c for c in self.conventions if c.status == "uncertain"]

    @property
    def ignored(self) -> list[AdaptedConvention]:
        return [c for c in self.conventions if c.status == "ignored"]


_LANGUAGE_KEYWORDS = {
    "python": ["python", ".py", "pyproject.toml"],
    "typescript": ["typescript", ".ts", "tsconfig"],
    "javascript": ["javascript", ".js", "js"],
    "go": ["go", ".go", "go.mod"],
    "rust": ["rust", ".rs", "cargo"],
    "java": ["java", ".java", "maven", "gradle"],
    "ruby": ["ruby", ".rb", "gemfile"],
    "php": ["php", ".php", "composer"],
    "csharp": ["csharp", ".cs", ".csproj"],
    "cpp": ["c++", ".cpp", ".hpp", "cmake"],
    "shell": ["shell", "bash", ".sh"],
}

_TOOL_KEYWORDS = {
    "ruff": ["ruff"],
    "black": ["black"],
    "mypy": ["mypy"],
    "prettier": ["prettier"],
    "eslint": ["eslint"],
    "gofmt": ["gofmt"],
    "golangci-lint": ["golangci"],
    "rustfmt": ["rustfmt"],
    "clippy": ["clippy"],
}

_TEST_KEYWORDS = {
    "pytest": ["pytest"],
    "unittest": ["unittest"],
    "jest": ["jest"],
    "vitest": ["vitest"],
    "go test": ["go test"],
    "cargo test": ["cargo test"],
    "rspec": ["rspec"],
}

_CATEGORY_KEYWORDS = {
    "directory_structure": [
        "directory",
        "structure",
        "src/",
        "tests/",
        "apps/",
        "packages/",
    ],
    "testing": ["test", "testing", "pytest", "jest", "unittest", "vitest", "rspec"],
    "formatter": [
        "format",
        "formatter",
        "ruff",
        "black",
        "prettier",
        "gofmt",
        "rustfmt",
    ],
    "linter": ["lint", "linter", "ruff", "eslint", "golangci", "clippy"],
    "type_checker": ["type", "mypy", "typescript"],
    "git": ["git", "commit", "branch", "merge"],
}


def split_markdown_sections(content: str) -> list[ReferenceSection]:
    import re

    lines = content.splitlines()
    sections: list[ReferenceSection] = []
    current_heading = ""
    current_level = 0
    current_body: list[str] = []

    for line in lines:
        m = re.match(r"^(#{1,6})\s+(.*)$", line)
        if m:
            if current_body or current_heading:
                sections.append(
                    ReferenceSection(
                        heading=current_heading,
                        body="\n".join(current_body).strip(),
                        level=current_level,
                    )
                )
            current_level = len(m.group(1))
            current_heading = m.group(2)
            current_body = []
        else:
            current_body.append(line)

    if current_body or current_heading:
        sections.append(
            ReferenceSection(
                heading=current_heading,
                body="\n".join(current_body).strip(),
                level=current_level,
            )
        )
    elif not sections:
        sections.append(ReferenceSection(heading="", body=content, level=0))

    return sections


def _detect_category(section: ReferenceSection) -> str:
    text = (section.heading + " " + section.body).lower()

    for category, keywords in _CATEGORY_KEYWORDS.items():
        for kw in keywords:
            if kw.lower() in text:
                return category

    return "unknown"


def _extract_languages(text: str) -> set[str]:
    found = set()
    text_lower = text.lower()

    for lang, keywords in _LANGUAGE_KEYWORDS.items():
        for kw in keywords:
            if kw.lower() in text_lower:
                found.add(lang)
                break

    return found


def _extract_tools(text: str) -> set[str]:
    found = set()
    text_lower = text.lower()

    for tool, keywords in _TOOL_KEYWORDS.items():
        for kw in keywords:
            if kw.lower() in text_lower:
                found.add(tool)
                break

    for tool, keywords in _TEST_KEYWORDS.items():
        for kw in keywords:
            if kw.lower() in text_lower:
                found.add(tool)
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


def _target_tools(target_analysis: dict) -> set[str]:
    found: set[str] = set()
    config = target_analysis.get("config", {})

    for _lang_key, lang_data in config.items():
        if isinstance(lang_data, dict):
            for tool_type in ("formatter", "linter", "type_checker"):
                tool_info = lang_data.get(tool_type)
                if isinstance(tool_info, dict) and tool_info.get("name"):
                    found.add(tool_info["name"].lower())

    return found


def _target_test_frameworks(target_analysis: dict) -> set[str]:
    found: set[str] = set()
    tests = target_analysis.get("tests", {})

    if isinstance(tests, dict):
        for fw_info in tests.get("frameworks", []):
            if isinstance(fw_info, dict) and fw_info.get("name"):
                found.add(fw_info["name"].lower())

    return found


def _target_paths(target_analysis: dict) -> set[str]:
    found: set[str] = set()
    tree = target_analysis.get("scan", {}).get("tree", [])

    for entry in tree:
        path = entry.get("path", "")
        if path:
            found.add(path)

    return found


def _check_directory_paths(
    section: ReferenceSection, target_analysis: dict
) -> tuple[str, str]:
    target_paths = _target_paths(target_analysis)
    body = section.body

    import re

    referenced = re.findall(r"[\w\-]+", body)
    referenced = [r for r in referenced if len(r) > 1]
    matched = [p for p in referenced if any(p in t.split("/") for t in target_paths)]

    if matched:
        return (
            "applicable",
            f"referenced paths found in target: {', '.join(matched[:3])}",
        )

    if referenced:
        return "mismatched", "referenced paths not found in target scan tree"

    return "uncertain", "no directory paths referenced"


def _classify_section(
    section: ReferenceSection, target_analysis: dict
) -> AdaptedConvention:
    category = _detect_category(section)
    text = (section.heading + " " + section.body).lower()

    if category == "directory_structure":
        status, reason = _check_directory_paths(section, target_analysis)

        return AdaptedConvention(
            section=section,
            category=category,
            status=status,
            reason=reason,
        )

    if category == "git":
        git_data = target_analysis.get("git")

        if git_data:
            return AdaptedConvention(
                section=section,
                category=category,
                status="applicable",
                reason="git analysis detected in target",
            )

        return AdaptedConvention(
            section=section,
            category=category,
            status="uncertain",
            reason="target analysis missing git data",
        )

    ref_languages = _extract_languages(text)
    ref_tools = _extract_tools(text)

    if ref_languages:
        target_langs = _target_languages(target_analysis)

        if target_langs:
            overlap = ref_languages & target_langs
            if overlap:
                return AdaptedConvention(
                    section=section,
                    category="language",
                    status="applicable",
                    reason=f"language {overlap.pop()} found in target scan summary",
                )

            return AdaptedConvention(
                section=section,
                category="language",
                status="mismatched",
                reason=f"language {ref_languages.pop()} not found in target scan summary",
            )

        return AdaptedConvention(
            section=section,
            category="language",
            status="uncertain",
            reason="target analysis missing scan summary languages",
        )

    if ref_tools:
        target_tools = _target_tools(target_analysis)
        target_tests = _target_test_frameworks(target_analysis)
        all_target_tools = target_tools | target_tests

        if all_target_tools:
            overlap = ref_tools & all_target_tools
            if overlap:
                return AdaptedConvention(
                    section=section,
                    category=category if category != "unknown" else "tool",
                    status="applicable",
                    reason=f"tool {overlap.pop()} detected in target analysis",
                )

            return AdaptedConvention(
                section=section,
                category=category if category != "unknown" else "tool",
                status="mismatched",
                reason=f"tool {ref_tools.pop()} mentioned but not detected in target",
            )

        return AdaptedConvention(
            section=section,
            category=category if category != "unknown" else "tool",
            status="uncertain",
            reason="target analysis missing config data",
        )

    return AdaptedConvention(
        section=section,
        category="unknown",
        status="uncertain",
        reason="no recognizable language, tool, or directory keywords",
    )


def adapt_reference(
    document: ReferenceDocument, target_analysis: dict
) -> ReferenceAdaptationResult:
    sections = split_markdown_sections(document.content)
    conventions = [_classify_section(s, target_analysis) for s in sections]

    return ReferenceAdaptationResult(source=document.source, conventions=conventions)


def adapt_references(
    documents: list[ReferenceDocument],
    target_analysis: dict,
) -> list[ReferenceAdaptationResult]:
    return [adapt_reference(d, target_analysis) for d in documents]
