# SYSTEM.md: Agentskill LLM Generation Contract

This document is the behavioral source of truth for generated `AGENTS.md`
files. Generated guidance must be grounded in repository evidence and must
allow an agent to produce code consistent with the analyzed repository.

## Output Contract

The LLM is the only author of semantic `AGENTS.md` content. The Rust CLI
provides deterministic evidence and read-only validation; it does not generate
or update Markdown.

The operational `AGENTS.md` must stay compact and organize guidance around
agent decisions:

1. Mission and repository map
2. Non-negotiables
3. Don’ts
4. Quick-start commands
5. Change routing
6. Architecture rules
7. Implementation conventions
8. Testing and validation
9. Common change playbooks
10. Further context

Target 500–1,000 tokens and warn above 1,500 tokens. Omit sections that have no
useful evidence. Scope every rule to the repository, package, service, or
language where it applies. Keep the Don’ts section to three to seven concrete,
high-impact items.

Every imperative rule must be backed by repository evidence or an explicit
maintainer answer. Preserve concrete examples when they are necessary to make
a rule operational, but do not include raw analyzer statistics or duplicated
rationale.

`AGENTS.md` must reference `AGENTS.reference.md` when that file exists. The
operational document must remain safe and useful if the reference is not read.

`AGENTS.reference.md` contains detailed architecture, ownership, workflows,
testing topology, rationale, evidence, examples, history, and uncertainty. It
may be larger than the operational document, but large repositories must split
it into topic files and load only relevant context.

## Generation Modes

The packaged skill provides the complete LLM workflow. The model reads the
evidence bundle, representative source files, configuration, tests, Git
history, and this specification before writing the final document.

The skill supports `init`, `enrich`, `scope`, `context`, `update`, and `audit`
workflows. `operational` and `reference` are depth views over one repository
understanding, not competing sources of truth.

## Quality Requirements

- Never invent commands, tools, file paths, or conventions.
- Prefer source-backed rules over analyzer statistics.
- Surface unresolved ambiguity or preserve existing manual text.
- Keep the root document self-sufficient and within its token budget.
- Use reference context for rationale instead of duplicating it in the root.
- Keep markdown headings, links, code fences, and trailing newline behavior
  valid and deterministic.
- Update this specification whenever generation semantics change.
