---
name: agentskill
description: Let any agent produce code consistent with the existing codebase.
---

# Agentskill Skill

Use the agentskill binary to gather repository evidence, then author and
maintain `AGENTS.md` through the LLM. Analyzer output is raw material, not the
finished document.

## Workflow

1. Confirm the target repository path.
2. Discover scopes with `agentskill scopes <repo> --pretty`, then collect
   normalized evidence with `agentskill evidence <repo> --pretty`.
3. Inspect representative entrypoints, core modules, tests, manifests, CI, and
   configuration files directly.
4. Read `SYSTEM.md` fully before drafting or updating documents.
5. Ask only high-impact questions that evidence cannot resolve.
6. Select an explicit scope and budget mode. Use `compact` for CPU-constrained
   local harnesses, `standard` by default, and `deep` only when broader context
   is justified.
7. Write compact managed `AGENTS.md` guidance and reference context when
   provenance or decisions require it. Scoped documents include local `## Scope`
   metadata, an independent `## Free Region`, and an explicit parent link.
8. Treat legacy scoped documents as read-only advisory candidates. Adopt or
   migrate one only after explicit, reviewed LLM workflow selection; never
   rewrite it automatically.
9. Present the complete semantic plan and diff before writing. Never create
   documents for merely suggested candidates.
10. Run `agentskill validate <repo> --signature <mode>` and show the semantic
   diff. Use `auto` unless the workflow request explicitly selects `on` or
   `off`.

Use individual analyzers when a focused signal is needed:

```bash
agentskill scan <repo> --pretty
agentskill measure <repo> --pretty
agentskill config <repo> --pretty
agentskill git <repo> --pretty
agentskill graph <repo> --pretty
agentskill symbols <repo> --pretty
agentskill tests <repo> --pretty
agentskill scopes <repo> --pretty
agentskill evidence <repo> --scope packages/api --budget compact --pretty
```

References are explicit inputs to the LLM workflow and must contain a readable
`AGENTS.md`. Compare reference conventions against target-repository evidence;
never copy them as unquestioned truth.

## LLM Workflows

Use these workflows for semantic repository guidance:

```bash
agentskill evidence <repo> --pretty
agentskill validate <repo>
agentskill drift <repo>
```

The LLM skill owns `init`, `enrich`, `scope`, `update`, `audit`, and
`explain`. The Rust CLI never writes semantic Markdown.

### `init`

Collect evidence, inspect representative files, ask one small batch of
unresolved high-impact questions, and create the canonical managed documents.
Use `## Free Region` for maintainer customs, add the managed signature unless
the resolved mode is `off`, and record provenance and decisions in the
reference document. Include the Agentskill version, evidence schema version,
repository revision, configuration source, maintainer decisions, and
unresolved uncertainty in its `## Provenance And Decisions` section. The
Agentskill version controls freshness; the repository revision records exact
provenance and may change without making guidance stale.

### `scope`

Discover existing nested documents and high-confidence package, workspace, or
service boundaries. Show parent relationships and missing-document suggestions.
Create or adopt a scope only after the maintainer explicitly selects its
repository-relative path. Follow the reported ancestor chain and nearest
managed fallback. Keep shared rules at their nearest owner, and resolve
conflicting inherited rules explicitly before writing.

### `update`

Compare current evidence with the managed documents, identify affected
canonical sections, and update only Agentskill-owned content. Preserve the
complete `## Free Region` body verbatim. Reconcile the exact managed signature,
refresh the visible provenance fields, run validation with the same signature
mode, and show a semantic diff.

### `audit`

Make no document changes. Report stale Agentskill versions, changed repository
revisions, unsupported or low-confidence rules, contradictions, broken
references, malformed signatures, configuration problems, and unresolved
uncertainty with their evidence paths and fact IDs. A changed repository
revision is informational unless the Agentskill version is stale. Treat
unsupported facts as errors and inferred or uncertain facts as warnings.

### `explain`

Make no document changes. Given a selected rule, explain its supporting fact
IDs, source paths, confidence, repository revision, and maintainer decision or
uncertainty.

All generation and maintenance workflows accept `budget compact|standard|deep`
with fixed input, output, and follow-up ceilings defined in `SYSTEM.md`.

## Evidence Rules

- Extract rules from source and configuration; do not guess.
- Treat analyzer counts as evidence, not prose for the final document.
- Scope every rule to the repository, service, or target language where it
  applies.
- Surface genuine uncertainty instead of silently inventing conventions.
- Keep supported target languages unchanged, including Python, while keeping
  the agentskill implementation Rust-only.

Read `SYSTEM.md` for the complete generated-document contract.
