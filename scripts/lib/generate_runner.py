"""Direct AGENTS.md generation workflow without merge/update semantics."""

from pathlib import Path

from common.fs import validate_repo
from lib.logging_utils import get_logger
from lib.output import validate_out_path
from lib.reference_flow import load_reference_documents
from lib.reference_initialization import (
    initialize_from_references,
    render_reference_metadata_block,
)
from lib.runner import run_all
from lib.update_feedback import load_feedback
from lib.update_merge import merge_agents_document
from lib.update_runner import DOCUMENT_TITLE, SECTION_ORDER, render_agents_sections


def _inject_reference_metadata(markdown: str, metadata_block: str) -> str:
    if markdown.startswith(DOCUMENT_TITLE):
        return (
            DOCUMENT_TITLE
            + metadata_block
            + "\n\n"
            + markdown.removeprefix(DOCUMENT_TITLE)
        )

    return metadata_block + "\n\n" + markdown


def render_agents_markdown(repo: Path, *, references: list[str] | None = None) -> str:
    documents = load_reference_documents(references)
    analysis = run_all(str(repo))
    feedback = load_feedback(repo)
    sections = render_agents_sections(repo, analysis, feedback)

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
) -> int:
    logger = get_logger()

    try:
        repo_path = validate_repo(repo)
        markdown = render_agents_markdown(repo_path, references=references)

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
