"""Direct AGENTS.md generation workflow without merge/update semantics."""

from pathlib import Path

from agentskill.common.fs import validate_repo
from agentskill.lib.interactive_runner import (
    PromptIO,
    StdinPromptIO,
    apply_interactive_notes,
    ask_generation_questions,
    detect_generation_gaps,
    interactive_section_notes,
)
from agentskill.lib.logging_utils import get_logger
from agentskill.lib.output import validate_out_path
from agentskill.lib.reference_flow import load_reference_documents
from agentskill.lib.reference_initialization import (
    initialize_from_references,
    render_reference_metadata_block,
)
from agentskill.lib.runner import run_all
from agentskill.lib.update_feedback import load_feedback
from agentskill.lib.update_merge import merge_agents_document
from agentskill.lib.update_runner import (
    DOCUMENT_TITLE,
    SECTION_ORDER,
    render_agents_sections,
)


def _inject_reference_metadata(markdown: str, metadata_block: str) -> str:
    if markdown.startswith(DOCUMENT_TITLE):
        return (
            DOCUMENT_TITLE
            + metadata_block
            + "\n\n"
            + markdown.removeprefix(DOCUMENT_TITLE)
        )

    return metadata_block + "\n\n" + markdown


def render_agents_markdown(
    repo: Path,
    *,
    references: list[str] | None = None,
    interactive: bool = False,
    prompt_io: PromptIO | None = None,
) -> str:
    documents = load_reference_documents(references)
    analysis = run_all(str(repo))
    feedback = load_feedback(repo)
    sections = render_agents_sections(repo, analysis, feedback)

    if interactive:
        gaps = detect_generation_gaps(analysis, documents)
        answers = ask_generation_questions(gaps, prompt_io or StdinPromptIO())
        sections = apply_interactive_notes(sections, interactive_section_notes(answers))

    result = merge_agents_document(
        None,
        sections,
        force=True,
        document_preamble=DOCUMENT_TITLE,
        preferred_order=SECTION_ORDER,
    )

    markdown = result.text

    if not references:
        return markdown

    initialization = initialize_from_references(analysis, documents)
    metadata_block = render_reference_metadata_block(initialization.metadata)

    return _inject_reference_metadata(markdown, metadata_block)


def generate_agents(
    repo: str,
    *,
    out: str | None = None,
    references: list[str] | None = None,
    interactive: bool = False,
    prompt_io: PromptIO | None = None,
) -> int:
    logger = get_logger()

    try:
        repo_path = validate_repo(repo)
        markdown = render_agents_markdown(
            repo_path,
            references=references,
            interactive=interactive,
            prompt_io=prompt_io,
        )

        if out is not None:
            output_path = validate_out_path(out)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(markdown)
        else:
            print(markdown, end="")
    except Exception as exc:
        logger.error("Generate failed for repo %s: %s", repo, exc)
        return 1

    return 0
