"""Internal workflow for updating AGENTS.md from current analyzer output."""

from collections import Counter
from pathlib import Path

from common.fs import read_text, validate_repo
from lib.agents_document import AgentsSection, build_section, normalize_section_name
from lib.logging_utils import get_logger
from lib.output import validate_out_path
from lib.runner import run_all
from lib.update_merge import merge_agents_document

AGENTS_FILENAME = "AGENTS.md"
DOCUMENT_TITLE = "# AGENTS\n\n"

SECTION_ORDER = [
    "overview",
    "repository structure",
    "service map",
    "cross-service boundaries",
    "commands and workflows",
    "code formatting",
    "naming conventions",
    "type annotations",
    "imports",
    "error handling",
    "comments and docstrings",
    "testing",
    "git",
    "dependencies and tooling",
    "red lines",
]

SECTION_HEADINGS = {
    "overview": "1. Overview",
    "repository structure": "2. Repository Structure",
    "service map": "3. Service Map",
    "cross-service boundaries": "4. Cross-Service Boundaries",
    "commands and workflows": "5. Commands and Workflows",
    "code formatting": "6. Code Formatting",
    "naming conventions": "7. Naming Conventions",
    "type annotations": "8. Type Annotations",
    "imports": "9. Imports",
    "error handling": "10. Error Handling",
    "comments and docstrings": "11. Comments and Docstrings",
    "testing": "12. Testing",
    "git": "13. Git",
    "dependencies and tooling": "14. Dependencies and Tooling",
    "red lines": "15. Red Lines",
}


def _format_languages(scan: dict) -> str:
    by_language = scan.get("summary", {}).get("by_language", {})
    languages = sorted(by_language)

    if not languages:
        return "No primary language could be determined from the repository scan."

    if len(languages) == 1:
        return languages[0]

    return ", ".join(languages[:-1]) + f", and {languages[-1]}"


def _top_level_layout(scan: dict) -> list[str]:
    tree = scan.get("tree", [])
    grouped: dict[str, list[str]] = {}

    for entry in tree:
        path = entry.get("path", "")

        if not path:
            continue

        head = path.split("/", 1)[0]
        grouped.setdefault(head, []).append(path)

    lines: list[str] = []

    for name in sorted(grouped):
        suffix = "/" if any("/" in path for path in grouped[name]) else ""
        kind = "test files" if name == "tests" else "source files"
        lines.append(f"{name}{suffix}  # {kind} ({len(grouped[name])} files)")

    return lines


def _python_commands(config: dict, tests: dict) -> list[str]:
    commands = ["pip install -e ."]
    python_tests = tests.get("python", {})
    run_command = python_tests.get("run_command")

    if run_command:
        commands.append(run_command)

    python_config = config.get("python", {})
    linter = python_config.get("linter", {}).get("name")

    if linter == "ruff":
        commands.extend(["ruff format .", "ruff check ."])

    type_checker = python_config.get("type_checker", {}).get("name")

    if type_checker == "mypy":
        commands.append("mypy")

    return commands


def _render_overview(repo: Path, analysis: dict) -> str:
    scan = analysis.get("scan", {})
    graph = analysis.get("graph", {})
    boundaries = graph.get("monorepo_boundaries", {})
    languages = _format_languages(scan)
    architecture = "monorepo" if boundaries.get("detected") else "single repository"

    return (
        f"{repo.name} is a {architecture} codebase analyzed by agentskill. "
        f"The primary language set detected here is {languages}, and the current "
        "AGENTS.md update flow is driven from analyzer output rather than manual editing.\n"
    )


def _render_repository_structure(analysis: dict) -> str:
    scan = analysis.get("scan", {})
    lines = _top_level_layout(scan)
    body = [
        "```text",
        *lines,
        "```",
        "",
    ]

    if "tests/" in "\n".join(lines):
        body.append(
            "- Keep tests under `tests/`; this repo separates tests from source."
        )

    source_roots = [
        line.split("  #", 1)[0] for line in lines if not line.startswith("tests")
    ]

    if source_roots:
        body.append(
            f"- Keep new source files under existing roots such as `{source_roots[0]}`."
        )

    return "\n".join(body) + "\n"


def _render_service_map(analysis: dict) -> str | None:
    boundaries = analysis.get("graph", {}).get("monorepo_boundaries", {})
    services = boundaries.get("services", [])

    if not boundaries.get("detected") or not services:
        return None

    lines = []

    for service in services:
        lines.append(f"### {service}")
        lines.append(f"- Service root: `{service}`")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def _render_cross_service_boundaries(analysis: dict) -> str | None:
    boundaries = analysis.get("graph", {}).get("monorepo_boundaries", {})

    if not boundaries.get("detected"):
        return None

    imports = boundaries.get("cross_service_imports", [])

    if imports:
        return (
            "- Cross-service imports were detected in the dependency graph.\n"
            "- Review shared contracts before changing any service boundary.\n"
        )

    return (
        "- No cross-service imports were detected in the current graph analysis.\n"
        "- Preserve service boundaries unless a shared contract layer is introduced.\n"
    )


def _render_commands_and_workflows(analysis: dict) -> str:
    commands = _python_commands(
        analysis.get("config", {}),
        analysis.get("tests", {}),
    )
    return "```bash\n" + "\n".join(commands) + "\n```\n"


def _render_code_formatting(analysis: dict) -> str:
    python_metrics = analysis.get("measure", {}).get("python", {})

    if not python_metrics:
        return "No formatting metrics were extracted from the current analysis run.\n"

    indentation = python_metrics.get("indentation", {})
    line_length = python_metrics.get("line_length", {})
    blank_lines = python_metrics.get("blank_lines", {})

    return (
        "### Python\n\n"
        f"- Indentation: `{indentation.get('size', 0)}` "
        f"{indentation.get('unit', 'unknown')}\n"
        f"- Line length: observed p95 is `{line_length.get('p95', 0)}`\n"
        f"- Blank lines after imports: mode `{blank_lines.get('after_imports', {}).get('mode', 0)}`\n"
        f"- Trailing newline: present in `{python_metrics.get('trailing_newline', {}).get('present', 0)}` files\n"
        f"- Trailing whitespace: present in `{python_metrics.get('trailing_whitespace', {}).get('files_with_trailing_ws', 0)}` files\n"
    )


def _render_naming_conventions(analysis: dict) -> str:
    symbols = analysis.get("symbols", {}).get("python", {})
    function_patterns = ", ".join(
        sorted(symbols.get("functions", {}).get("patterns", {}))
    )

    class_patterns = ", ".join(sorted(symbols.get("classes", {}).get("patterns", {})))
    constant_patterns = ", ".join(
        sorted(symbols.get("constants", {}).get("patterns", {}))
    )

    return (
        "### Python\n\n"
        f"- Functions: `{function_patterns or 'unknown'}`\n"
        f"- Classes: `{class_patterns or 'unknown'}`\n"
        f"- Constants: `{constant_patterns or 'unknown'}`\n"
        "- Match existing file names and test names rather than introducing a new naming scheme.\n"
    )


def _python_source_paths(scan: dict) -> list[str]:
    return [
        entry.get("path", "")
        for entry in scan.get("tree", [])
        if entry.get("language") == "python"
    ]


def _render_type_annotations(repo: Path, analysis: dict) -> str:
    scan = analysis.get("scan", {})
    paths = _python_source_paths(scan)
    annotated = 0
    total_defs = 0

    for rel_path in paths:
        content = read_text(repo / rel_path)

        for line in content.splitlines():
            stripped = line.strip()

            if not stripped.startswith("def "):
                continue

            total_defs += 1

            if "->" in stripped or ":" in stripped.split("(", 1)[1]:
                annotated += 1

    config = analysis.get("config", {}).get("python", {})
    type_checker = config.get("type_checker", {}).get("name")

    return (
        "### Python\n\n"
        f"- Annotated function signatures detected: `{annotated}` of `{total_defs}` observed definitions.\n"
        f"- Type checker: `{type_checker or 'not detected'}`.\n"
        "- Match the local annotation density instead of introducing stricter typing patterns automatically.\n"
    )


def _first_import_block(repo: Path, analysis: dict) -> str:
    scan = analysis.get("scan", {})

    for rel_path in scan.get("read_order", []):
        content = read_text(repo / rel_path)
        lines: list[str] = []

        for line in content.splitlines():
            stripped = line.strip()

            if stripped.startswith("import ") or stripped.startswith("from "):
                lines.append(line)
                continue

            if lines and not stripped:
                lines.append(line)
                continue

            if lines:
                break

        if lines:
            return "\n".join(lines).rstrip()

    return ""


def _render_imports(repo: Path, analysis: dict) -> str:
    block = _first_import_block(repo, analysis)

    if not block:
        return "No representative import block was found in the scanned files.\n"

    return "```python\n" + block + "\n```\n"


def _render_error_handling(repo: Path, analysis: dict) -> str:
    scan = analysis.get("scan", {})
    handlers = 0
    raises = 0

    for rel_path in scan.get("read_order", []):
        content = read_text(repo / rel_path)
        handlers += content.count("except ")
        raises += content.count("raise ")

    return (
        "### Python\n\n"
        f"- `except` blocks observed across representative files: `{handlers}`\n"
        f"- `raise` statements observed across representative files: `{raises}`\n"
        "- Preserve the local error-handling style instead of introducing new exception patterns opportunistically.\n"
    )


def _render_comments_and_docstrings(repo: Path, analysis: dict) -> str:
    scan = analysis.get("scan", {})
    docstrings = 0
    comments = 0

    for rel_path in scan.get("read_order", []):
        content = read_text(repo / rel_path)
        docstrings += content.count('"""')

        for line in content.splitlines():
            if line.strip().startswith("#"):
                comments += 1

    return (
        "### Python\n\n"
        f"- Triple-quoted docstring markers observed: `{docstrings}`\n"
        f"- Line comments observed: `{comments}`\n"
        "- Keep comments sparse and prefer short docstrings when the file already uses them.\n"
    )


def _render_testing(analysis: dict) -> str:
    python_tests = analysis.get("tests", {}).get("python", {})
    coverage = python_tests.get("coverage_shape", {})
    fixtures = python_tests.get("fixtures", {})

    return (
        "### Python\n\n"
        f"- Framework: `{python_tests.get('framework', 'unknown')}`\n"
        f"- Run command: `{python_tests.get('run_command', 'unknown')}`\n"
        f"- Test file pattern: `{python_tests.get('naming', {}).get('file_pattern', 'unknown')}`\n"
        f"- Fixtures detected: `{', '.join(fixtures.get('fixture_names', [])) or 'none'}`\n"
        f"- Untested source files: `{len(coverage.get('untested_source_files', []))}`\n"
    )


def _render_git(analysis: dict) -> str:
    git = analysis.get("git", {})

    if "error" in git:
        return f"- Git analysis unavailable: `{git['error']}`.\n"

    commits = git.get("commits", {})
    prefixes = commits.get("prefixes", {})
    prefix_names = ", ".join(sorted(prefixes)) or "unknown"
    merge_strategy = git.get("merge_strategy", {}).get("detected", "unknown")

    return (
        f"- Commit prefixes observed: `{prefix_names}`.\n"
        f"- Merge strategy: `{merge_strategy}`.\n"
    )


def _render_dependencies_and_tooling(analysis: dict) -> str:
    config = analysis.get("config", {})
    tools: list[str] = []

    for lang_data in config.values():
        if not isinstance(lang_data, dict):
            continue

        for tool_type in ("formatter", "linter", "type_checker"):
            tool_info = lang_data.get(tool_type)

            if isinstance(tool_info, dict) and tool_info.get("name"):
                tools.append(tool_info["name"])

    counts = Counter(tools)
    tool_list = ", ".join(sorted(counts)) or "none detected"

    return f"- Tooling detected from config: `{tool_list}`.\n"


def _render_red_lines(analysis: dict) -> str:
    scan = analysis.get("scan", {})
    roots = ", ".join(
        sorted({path.split("/", 1)[0] for path in scan.get("read_order", [])})
    )

    return (
        "- Do not invent new top-level layout patterns when the scan already shows established roots.\n"
        f"- Keep changes aligned with existing areas such as `{roots or 'the detected source tree'}`.\n"
        "- Do not assume missing analyzer signals imply permission to rewrite local conventions.\n"
    )


def render_agents_sections(repo: Path, analysis: dict) -> dict[str, AgentsSection]:
    rendered: dict[str, str | None] = {
        "overview": _render_overview(repo, analysis),
        "repository structure": _render_repository_structure(analysis),
        "service map": _render_service_map(analysis),
        "cross-service boundaries": _render_cross_service_boundaries(analysis),
        "commands and workflows": _render_commands_and_workflows(analysis),
        "code formatting": _render_code_formatting(analysis),
        "naming conventions": _render_naming_conventions(analysis),
        "type annotations": _render_type_annotations(repo, analysis),
        "imports": _render_imports(repo, analysis),
        "error handling": _render_error_handling(repo, analysis),
        "comments and docstrings": _render_comments_and_docstrings(repo, analysis),
        "testing": _render_testing(analysis),
        "git": _render_git(analysis),
        "dependencies and tooling": _render_dependencies_and_tooling(analysis),
        "red lines": _render_red_lines(analysis),
    }

    sections: dict[str, AgentsSection] = {}

    for name in SECTION_ORDER:
        body = rendered.get(name)

        if body is None:
            continue

        sections[name] = build_section(
            SECTION_HEADINGS[name],
            body,
            heading_level=2,
        )

    return sections


def _resolve_update_path(repo: Path, out: str | None) -> Path:
    if out is None:
        return repo / AGENTS_FILENAME

    return validate_out_path(out)


def _validate_requested_sections(
    include_sections: list[str] | None,
    exclude_sections: list[str] | None,
    supported_sections: dict[str, AgentsSection],
) -> None:
    requested = {
        *[normalize_section_name(name) for name in include_sections or []],
        *[normalize_section_name(name) for name in exclude_sections or []],
    }
    unsupported = sorted(name for name in requested if name not in supported_sections)

    if unsupported:
        names = ", ".join(unsupported)
        raise ValueError(f"unsupported or unavailable sections: {names}")


def update_agents(
    repo: str,
    *,
    include_sections: list[str] | None = None,
    exclude_sections: list[str] | None = None,
    force: bool = False,
    out: str | None = None,
) -> int:
    """Update or create AGENTS.md for a repository."""
    logger = get_logger()

    try:
        repo_path = validate_repo(repo)
        analysis = run_all(str(repo_path))
        sections = render_agents_sections(repo_path, analysis)
        _validate_requested_sections(include_sections, exclude_sections, sections)
        target_path = _resolve_update_path(repo_path, out)
        existing_path = repo_path / AGENTS_FILENAME

        existing_text = (
            read_text(existing_path, None) if existing_path.exists() else None
        )

        merged = merge_agents_document(
            existing_text,
            sections,
            include_sections=include_sections,
            exclude_sections=exclude_sections,
            force=force,
            document_preamble=DOCUMENT_TITLE,
            preferred_order=SECTION_ORDER,
        )

        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_text(merged.text)
    except Exception as exc:
        logger.error("Update failed for repo %s: %s", repo, exc)
        return 1

    return 0
