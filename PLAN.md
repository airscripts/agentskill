# Guidance Plan

## Summary

Implement Milestone 1 of the 2.1 roadmap: make Agentskill reliable for
creating, enriching, validating, auditing, and updating tool-managed root
`AGENTS.md` documents while preserving one explicit maintainer-owned `Free
Region`.

The Rust runtime remains deterministic, read-only, and responsible only for
repository evidence, configuration loading, validation, and drift reporting.
The packaged LLM skill remains responsible for semantic Markdown authorship and
the `init`, `update`, `audit`, and `explain` workflows.

Success means that a repository can opt into a visible managed signature,
produce evidence with traceable provenance, safely update an existing document,
and run advisory drift checks without converting uncertain analyzer output into
rules or blocking CI by default.

## Status

Complete. Milestone 1 is implemented, documented, tested, and verified with
the locked workspace checks.

## Implementation Changes

### Repository Configuration

- Add a shared repository-configuration loader for the root `agentskill.toml`.
- Support one top-level setting: `signature = true | false`; default to `true`
  when the file is absent.
- Return the effective value, whether it came from the file or the default,
  and a parse error when the file is malformed.
- Treat malformed configuration as a validation error and as an explicit
  uncertainty in evidence; do not silently apply a non-default value.
- Keep workflow overrides ephemeral. A workflow may choose `on` or `off` for
  that run, but it must not modify `agentskill.toml`.

### Evidence And Provenance

- Preserve evidence schema version `3` and all existing analyzer keys and
  fields.
- Add repository configuration state to the evidence bundle, including the
  effective signature setting and its source (`default` or `agentskill.toml`).
- Keep stable fact identifiers, scope, confidence, repository revision, and
  evidence paths. Add signature configuration as a repository-scoped fact.
- Do not add a JSON sidecar or hidden Markdown metadata.
- Require generated reference context to contain a visible `Provenance And
  Decisions` section with the evidence schema version, repository revision,
  configuration source, and maintainer-confirmed decisions.
- Record a declined or unanswered high-impact question as uncertainty rather
  than as a maintainer decision.

### Managed Document Signature

Use this exact canonical footer, with a trailing newline:

```markdown
---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
```

- Apply the footer to generated root `AGENTS.md` and
  `AGENTS.reference.md` documents.
- Treat the footer as final content. Whitespace after it is allowed only as
  the document’s final newline.
- On every generated or updated document, replace at most one exact canonical
  footer, append it when enabled and absent, and remove it when disabled.
- Never replace or remove custom footer text. A recognizable but non-canonical
  Agentskill footer remains for validation to report as malformed.
- Detect and report missing, duplicated, malformed, or non-terminal canonical
  footers when signatures are enabled. Do not require a footer when the
  effective setting is disabled.
- Resolve signature mode in this order: explicit workflow/check override,
  repository configuration, default `true`.

### Document Ownership And Updates

- Define the canonical operational section IDs from `SYSTEM.md`: Mission And
  Repository Map, Non-Negotiables, Don’ts, Quick Start, Change Routing,
  Architecture Rules, Implementation Conventions, Testing And Validation,
  Common Change Playbooks, `## Free Region`, and Further Context.
- Define the reference sections for provenance, ownership, evidence
  interpretation, testing topology, rationale, history, and uncertainty.
- Match headings by normalized section name, while accepting the existing
  repository’s equivalent headings during migration.
- `init` creates the canonical root and reference documents, including exactly
  one visible `## Free Region` section in the root document.
- `AGENTS.md` and `AGENTS.reference.md` are tool-managed artifacts. The LLM
  skill may rewrite managed sections when evidence changes, and maintainers
  must place custom instructions in `## Free Region` if they need them
  preserved.
- Preserve the complete `## Free Region` body verbatim, including its nested
  headings, links, and formatting. Never generate, normalize, summarize,
  reorder, or delete content inside that region.
- Content outside `## Free Region`, including unknown headings and manual edits,
  is tool-owned and may be reconciled with the canonical document structure on
  `update` or `enrich`.
- Update managed sections in place where possible, but permit a complete
  managed-document rewrite when required to restore the canonical structure.
- Preserve custom footer text only when it is inside `## Free Region`; outside
  that region, only the canonical managed footer is retained.
- Existing documents are migrated into the managed structure. They are not
  treated as authoritative over current evidence.

### Interactive Review

- Before drafting or updating, inspect normalized evidence and representative
  source, configuration, CI, tests, and Git history.
- Ask at most one small batch of high-impact questions for ambiguity that
  repository inspection cannot resolve. Each question must state the evidence
  and the consequence of the choice.
- If questions are declined, continue with conservative non-rule defaults and
  record unresolved uncertainty in the reference document.
- Never ask for facts that can be verified from the repository.
- Store accepted answers as visible maintainer-confirmed decisions in reference
  Markdown and distinguish them from repository observations.

### Validation And Drift

- Keep `validate` and `drift` read-only and place all behavior in the
  `agentskill-validation` crate, not the CLI entrypoint.
- Continue checking document existence, duplicate normalized headings, local
  references, trailing newline, and the operational token budget.
- Validate canonical signature behavior using the effective configuration and
  check `agentskill.toml` syntax.
- Verify referenced local commands and configuration files only when safely
  possible from static repository contents; never execute referenced commands.
- Read visible provenance from the reference document and report when its
  repository revision differs from the current revision.
- Report contradictions between document signature claims and current
  `agentskill.toml`, plus unsupported or low-confidence rules when provenance
  identifies them.
- Return machine-readable findings with stable `kind`, `severity`, `document`,
  `path` or `fact`, and human-readable `message` fields. Preserve the existing
  top-level `valid`, `errors`, `warnings`, `stale`, and `issues` fields for
  compatibility.
- Keep drift advisory: a completed drift analysis exits `0` even when findings
  exist. Invalid input, missing required documents, or an analysis failure
  remains a command error. Do not add strict CI mode in this milestone.
- Add an `auto | on | off` signature option to validation checks. `auto` uses
  repository configuration; `on` and `off` are ephemeral workflow/check
  overrides and do not write configuration.
- Add reusable GitHub Actions in `agentskill-actions/drift/` and
  `agentskill-actions/validate/`. They accept a pinned Agentskill release and
  signature mode, install the matching binary, write a concise job summary, and
  expose a JSON report path. Drift is advisory; validate fails on invalid
  documents.
- Add the caller workflow in `.github/workflows/agentskill.yml` for the main
  workflow and manual dispatch. It runs the checked-out source for drift and
  validation, uploads both reports, and documents the reusable Actions for
  adopter repositories.

### LLM Skill Workflows

Update `agentskill-skill/SYSTEM.md` and `agentskill-skill/SKILL.md` to make
these workflows normative and provider-neutral:

- `init`: gather evidence, inspect representative files, ask unresolved
  questions, and create compact operational plus reference context.
- `update`: identify affected canonical sections from changed evidence, patch
  only those sections, preserve unknown/manual content, refresh provenance, and
  reconcile the managed footer.
- `audit`: make no document changes; report stale, unsupported,
  contradictory, malformed, and low-confidence guidance with its evidence.
- `explain`: explain a selected rule using its source paths, fact identifiers,
  confidence, revision, and any maintainer decision.

All workflows accept an optional signature mode of `auto`, `on`, or `off`, use
the precedence defined above, and pass the selected mode to post-write
validation. They must show the semantic document diff and validation result.
The workflows remain LLM operations, not Rust CLI subcommands.

## Public Interfaces And Compatibility

- Add the shared configuration API in the owning lower-level crate and expose
  only typed configuration state needed by analyzers and validation.
- Extend the evidence JSON without renaming or removing schema version `3`
  fields, analyzer keys, fact IDs, or provenance fields.
- Add `--signature <auto|on|off>` only to `validate` and `drift`; omission keeps
  current behavior through `auto`.
- Change drift’s finding exit behavior to advisory success while retaining
  process errors for invalid paths, missing `AGENTS.md`, and failed analysis.
- Add the reusable `agentskill-actions/drift/` and
  `agentskill-actions/validate/` Actions and caller workflow; their inputs,
  outputs, JSON reports, and exit behavior are part of the release contract.
- Update CLI help, `agentskill-docs/cli.md`,
  `agentskill-docs/architecture.md`, `README.md`, and `CHANGELOG.md` for the
  configuration, signature, evidence, and drift contracts.
- Keep `agentskill/src/main.rs` as a binary shim. Put configuration and
  validation logic in core/analyzers/validation and dispatch only in the CLI
  library.

## Test Plan And Acceptance Criteria

### Rust Tests

- Configuration tests cover absent config, `signature = true`,
  `signature = false`, unknown keys policy, invalid TOML, and deterministic
  source reporting.
- Evidence contract tests cover configuration state, signature facts, stable
  provenance, deterministic ordering, and malformed-config uncertainty.
- Signature tests cover insertion for both generated document types, disabled
  validation, exact canonical replacement, duplicate detection, non-terminal
  detection, malformed detection, custom-footer preservation, and removal on
  opt-out.
- Update/ownership fixtures cover creation of `## Free Region`, verbatim
  preservation of its body and nested headings, managed updates outside it,
  migration of unknown/manual content outside it, reference-link preservation,
  token-budget preservation, and avoiding unnecessary footer rewrites.
- Validation tests cover broken paths, referenced commands/configuration files,
  stale revisions, contradictory configuration, unsupported/low-confidence
  provenance, and malformed configuration.
- CLI tests cover `--signature auto|on|off`, both binary names, JSON output, and
  drift returning success when advisory findings exist.
- Workflow checks cover pull-request and manual drift execution, pinned binary
  installation, JSON artifact publication, job-summary output, Action outputs,
  and successful completion with advisory findings.

### Skill Fixtures And End-To-End Checks

- Add fixtures for a new repository, an existing manually authored root
  document, an existing reference document, signatures enabled/disabled, a
  custom footer, duplicate/malformed footers, and declined questions.
- Assert that `init` and `update` produce canonical signatures and visible
  provenance, while `audit` and `explain` do not write files.
- Run validation after every write workflow and compare the semantic diff,
  including an unchanged-document case where no footer rewrite occurs.
- Use repository examples and target-language fixtures only as evidence inputs;
  do not add non-Rust runtime code.

### Completion Criteria

The milestone is complete when all required behaviors above have owning-crate
tests, the skill contract and user documentation agree with the implementation,
advisory drift checks exit successfully with findings, and the locked workspace
verification passes:

```bash
make fmt
make lint
make check
make test
make verify
```

## Assumptions And Defaults

- This plan covers only Milestone 1, Guidance. Task Context and
  Scoped Guidance remain roadmap milestones and are not implemented here.
- `agentskill.toml` is repository-level, root-only, and intentionally small.
- The default signature setting is enabled.
- Workflow/check overrides are temporary and never mutate repository config.
- `## Free Region` is the only maintainer-owned content area; all other document
  content is managed by Agentskill.
- The exact canonical footer is a compatibility contract.
- Reference provenance is visible Markdown, not a sidecar or hidden metadata.
- Drift is advisory by default and has no strict mode in this milestone.
- The current Rust workspace, schema version `3`, supported-language matrix,
  analyzer keys, and read-only CLI boundary remain the foundation.
