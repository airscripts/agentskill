# Agentskill Plan

## Summary

Position Agentskill as the default lifecycle tool for creating and maintaining
trustworthy `AGENTS.md` files.

The primary audience is individual developers. The primary promise is:

> Keep every repository’s `AGENTS.md` accurate, compact, explainable, and
> useful to coding agents.

Distribution remains provider-neutral through the local CLI and installable
LLM skill.

## Product Lifecycle

```text
Inspect -> Generate -> Validate -> Review -> Maintain -> Measure
```

The Rust CLI remains deterministic and read-only. It collects evidence,
normalizes facts and provenance, validates documents, detects drift, and emits
machine-readable reports. The LLM skill remains responsible for semantic
document authorship.

## Managed Document Signature

Every generated agent document receives this footer by default:

```markdown
---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this footer. It is automatically managed by Agentskill.
```

This applies to the root `AGENTS.md`, `AGENTS.reference.md`, and future scoped
`AGENTS.md` files.

### Opt-Out Configuration

Add an optional repository-level `agentskill.toml` file:

```toml
signature = false
```

Behavior:

- The default is `signature = true`.
- The setting applies consistently to all generated agent documents.
- An explicit workflow-level choice overrides the repository default for that
  run.
- When disabled, the footer is not added.
- An existing Agentskill footer is removed when the repository opts out.
- Custom, non-Agentskill footers are preserved.
- Validation does not require a footer when the setting is disabled.
- Opting out does not affect evidence collection, generation, updates, auditing,
  or drift checks.

Signature rules:

- The footer is final content when enabled.
- Only the canonical Agentskill footer is automatically replaced.
- The LLM must not rewrite or customize the footer.
- Validation detects missing, duplicated, malformed, or non-terminal footers.
- The footer uses no emojis or em dashes.
- The signature is an awareness feature, not a compliance requirement.

## Milestone 1: Trusted Maintenance

### Evidence And Provenance

Extend the evidence contract with stable fact identifiers, evidence paths,
scope, confidence, repository revision, maintainer-confirmed decisions, and
signature configuration state.

Store provenance in `AGENTS.reference.md` or topic references. Do not add a JSON
sidecar or hidden Markdown metadata.

### Document Ownership

Use structured canonical sections in `AGENTS.md`. The update workflow must:

- Preserve unknown and manually authored sections.
- Update only recognized generated sections.
- Preserve maintainer text where possible.
- Avoid whole-document rewrites.
- Preserve the operational token budget and reference link.
- Add, update, or remove only the managed footer according to configuration.

Existing documents should be audited and enriched rather than replaced
automatically.

### Interactive Review

During generation or update, inspect evidence first and ask one small batch of
high-impact questions only for unresolved ambiguity. Explain the evidence and
consequence of each question, continue with conservative defaults when
questions are declined, and record answers as maintainer-confirmed decisions in
the reference document.

Never ask questions for facts already discoverable from the repository.

### Validation And Drift

Improve `validate` and `drift` to check:

- Broken paths and local references.
- Missing or duplicate sections.
- Operational token budget.
- Canonical footer correctness.
- `agentskill.toml` syntax and signature behavior.
- Referenced commands and configuration files where safely verifiable.
- Stale evidence revisions.
- Contradictions with current repository configuration.
- Unsupported or low-confidence rules when provenance is available.

Drift remains advisory by default. It emits machine-readable and human-readable
findings and exits successfully when analysis completes. An explicit strict
mode may be added later for repositories that want blocking CI policy.

### LLM Workflows

Formalize these skill workflows:

- `init`: create initial guidance.
- `update`: update affected sections.
- `audit`: identify stale, unsupported, or contradictory guidance.
- `explain`: explain the evidence behind a rule.

These remain LLM skill workflows, not Rust CLI subcommands.

## Milestone 2: Task Context

After maintenance is reliable, add task-specific context generation. The skill
should produce relevant files, architecture and ownership boundaries, applicable
rules, required tests and commands, known hazards, similar historical changes,
and unresolved uncertainty.

Add evidence-backed playbooks for APIs, dependencies, configuration, tests,
schemas, releases, and generated files.

## Milestone 3: Scoped Guidance

Keep the first implementation root-focused. Later support:

```text
AGENTS.md
packages/api/AGENTS.md
services/payments/AGENTS.md
```

Define scope discovery, inheritance, precedence, conflict reporting,
shared-rule deduplication, root fallback behavior, and consistent signature
configuration across generated files.

## Evaluation And Success Criteria

The primary metric is the number of repositories that generate guidance and
continue using update or audit workflows across later revisions.

Supporting metrics include maintainer acceptance with minimal editing, the
percentage of rules with evidence, valid command and path rate, stale or
unsupported rule count, operational token count, agent task success, and fewer
wrong-file, wrong-command, and architecture-boundary mistakes.

Create benchmark repositories with repeatable coding tasks. Judge generated
guidance by agent behavior, not only Markdown structure.

Measurement must be opt-in and privacy-preserving. The CLI must not require
hosted telemetry.

## Required Tests

Add coverage for:

- Default signature insertion in every generated document type.
- Repository-wide opt-out through `agentskill.toml`.
- Explicit workflow-level signature override.
- Removal of an existing managed footer after opt-out.
- Preservation of custom footers.
- Duplicate and non-terminal footer detection.
- Validation when the signature is disabled.
- Malformed `agentskill.toml`.
- Updates that avoid unnecessary footer rewrites.
- Advisory CI drift behavior.
- Safe enrichment of manually authored `AGENTS.md` files.

## Deferred Product Roads

Defer hosted dashboards, hosted repository analysis, automatic local
regeneration, broad team governance, enterprise policy enforcement, additional
language support without guidance-quality improvements, and static semantic
Markdown generation in Rust.

## Assumptions And Defaults

- The current 2.1 architecture remains the foundation.
- The local CLI and LLM skill are the primary product surface.
- Generation is autonomous by default, with a small interactive review round
  for ambiguity.
- CI drift checks are advisory by default.
- Maintainer content is preserved through structured document sections.
- Provenance is stored in reference Markdown.
- The root `AGENTS.md` is the first-class artifact.
- Scoped documents come after trusted root maintenance.
- The signature is visible by default but never mandatory.
- `agentskill.toml` is the repository-level configuration source.
- Explicit workflow choices take precedence over repository defaults.
