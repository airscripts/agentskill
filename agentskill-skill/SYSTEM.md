# SYSTEM.md: Agentskill LLM Generation Contract

This document is the behavioral source of truth for generated `AGENTS.md`
files. Generated guidance must be grounded in repository evidence and must
allow an agent to produce code consistent with the analyzed repository.

Agentskill manages generated guidance. Maintainers must put custom
instructions in the exact root-level `## Free Region` section; content outside
that section is managed and may be reconciled during updates.

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
10. Free Region
11. Further context

Target 500–1,000 tokens and warn above 1,500 tokens. Omit sections that have no
useful evidence. Scope every rule to the repository, package, service, or
language where it applies. Keep the Don’ts section to three to seven concrete,
high-impact items.

Every imperative rule must be backed by repository evidence or an explicit
maintainer answer. Preserve concrete examples when they are necessary to make
a rule operational, but do not include raw analyzer statistics or duplicated
rationale.

`AGENTS.md` must contain exactly one local `## Free Region` section and must
reference its local `AGENTS.reference.md` when that file exists. A scoped
document must also contain a compact managed `## Scope` section declaring its
repository-relative path, nearest parent scope, and additive inheritance. The
operational document must remain useful when its reference is not read; parent
guidance is inherited through the portable Agentskill hierarchy and is not
flattened into child documents. When a child repeats an inherited rule, keep
the nearest owner and remove the duplicate; when local guidance conflicts with
an inherited rule, surface the conflict and resolve it explicitly.

Generated workflows create or update `AGENTS.reference.md` when provenance or
maintainer decisions need to be recorded. It contains detailed architecture,
ownership, workflows, testing topology, rationale, evidence, examples, history,
and uncertainty. It may be larger than the operational document, but large
repositories must split it into topic files and load only relevant context.
When it exists, its root-level `## Provenance And Decisions` section must show
the Agentskill version, evidence schema version, repository revision,
configuration source, maintainer-confirmed decisions, and unresolved
uncertainty. The Agentskill version determines freshness; the repository
revision records exact source provenance and may change without making
guidance stale.

## Generation Modes

The packaged skill provides the complete LLM workflow. The model reads the
evidence bundle, representative source files, configuration, tests, Git
history, and this specification before writing the final document. It must
preserve `## Free Region` verbatim and must not ask maintainers to edit managed
sections directly.

The skill supports `init`, `enrich`, `scope`, `update`, `audit`, and
`explain` workflows. `operational` and `reference` are depth views over one
repository understanding, not competing sources of truth. Milestone 1
formalizes `init`, `update`, `audit`, and `explain`; the other workflows remain
compatible but are not expanded here.

## Managed Signature

Generated documents use this exact final footer by default:

```markdown
---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
```

The repository may set `signature = false` in root `agentskill.toml`. Each
workflow accepts an ephemeral `signature` mode of `auto`, `on`, or `off`; an
explicit mode overrides repository configuration for that run and never edits
the configuration file. Custom instructions belong in `## Free Region`, not in
the managed footer or other managed sections.

## Resource Budgets

Workflows accept an ephemeral `budget` mode. `standard` is the default and
preserves normal evidence depth. `compact` is for CPU-constrained local
harnesses and preserves high-confidence operational guidance before dropping
low-confidence details, examples, history, or deep references. `deep` permits
broader reference loading.

The fixed ceilings are:

| Mode | Input context | Output | Follow-up rounds |
| --- | ---: | ---: | ---: |
| `compact` | 4,000 tokens | 512 tokens | 1 |
| `standard` | 8,000 tokens | 1,000 tokens | 2 |
| `deep` | 16,000 tokens | 2,000 tokens | 4 |

If a mode cannot produce a valid document within its output ceiling, the
workflow must report insufficient budget instead of truncating guidance.

## Quality Requirements

- Never invent commands, tools, file paths, or conventions.
- Prefer source-backed rules over analyzer statistics.
- Surface unresolved ambiguity and preserve only the complete `## Free Region`
  verbatim. Content outside that section is managed by Agentskill.
- Keep the root document self-sufficient and within its token budget.
- Keep scoped operational documents within the same 500–1,000 token target and
  1,500-token hard ceiling.
- Use reference context for rationale instead of duplicating it in the root.
- Keep markdown headings, links, code fences, and trailing newline behavior
  valid and deterministic.
- Update this specification whenever generation semantics change.
