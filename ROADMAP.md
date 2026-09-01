# Agentskill 2.1 Roadmap

## Release Goal

Position Agentskill as the default lifecycle tool for creating and maintaining
trustworthy `AGENTS.md` files.

The primary promise is:

> Keep every repository’s `AGENTS.md` accurate, compact, explainable, and
> useful to coding agents.

The local Rust CLI remains deterministic and read-only. It collects evidence,
normalizes facts and provenance, validates documents, detects drift, and emits
machine-readable reports. The installable LLM skill remains responsible for
semantic Markdown authorship and document maintenance.

## Ownership Model

- Agentskill manages the generated `AGENTS.md` and
  `AGENTS.reference.md` documents.
- `## Free Region` is the only maintainer-owned document area. Its contents are
  preserved verbatim across updates.
- Maintainer edits outside `## Free Region` may be reconciled or replaced when
  Agentskill refreshes the managed document.
- The Rust runtime owns evidence and checks; the LLM skill owns semantic
  Markdown authorship.

## Release Principles

- Gather evidence before writing guidance.
- Keep the Rust runtime deterministic, bounded, read-only, and Rust-only.
- Preserve public analyzer, evidence, language, CLI, and validation contracts
  unless a compatibility change is intentional and documented.
- Roll out root guidance before scoped guidance.
- Keep diagnostics advisory by default; repositories may adopt stricter policy
  later.
- Store provenance in visible reference Markdown, never in hidden metadata or
  a sidecar file.
- Keep measurement opt-in and privacy-preserving, with no hosted telemetry
  requirement.
- Keep model-facing context bounded so local CPU-based harnesses can use the
  skill with compact, standard, or deep budget modes.

## Product Lifecycle

```text
Inspect -> Generate -> Validate -> Review -> Maintain -> Measure
```

## Milestone Tracking

### Milestone 1: Guidance

Status: Complete

Completed: 2026-08-30

Make root `AGENTS.md` documents reliable to create, validate, audit, and
update. The document is managed by Agentskill; maintainers receive one
explicit `## Free Region` for custom instructions that the tool preserves
verbatim.

Deliverables:

- Stable evidence identifiers, provenance, repository revisions, confidence,
  and signature configuration state.
- Repository-level `agentskill.toml` configuration with a visible managed
  signature enabled by default and an explicit opt-out.
- Safe document ownership, canonical sections, `## Free Region` preservation,
  reference provenance, and minimal managed updates.
- Interactive review for unresolved high-impact ambiguity, with visible
  maintainer-confirmed decisions and recorded uncertainty.
- Validation and advisory drift findings for broken references, stale
  evidence, contradictions, signatures, configuration, and unsupported rules.
- Formalized LLM workflows for `init`, `update`, `audit`, and `explain`.
- Reusable GitHub Actions in `agentskill-actions/drift/` and
  `agentskill-actions/validate/`, plus a caller workflow that publishes
  human-readable findings and machine-readable output.

Implementation plan: [`PLAN.md`](./PLAN.md).

### Milestone 2: Scoped Guidance

Status: Planned

Extend the root-focused model to repositories with nested guidance files and
resource-constrained local AI harnesses.

Deliverables:

- Deterministic scope discovery for existing documents and high-confidence
  package, workspace, and service markers.
- Explicit scope selection, local evidence with inherited support, and a
  portable additive hierarchy with nearest-scope override.
- Nested `AGENTS.md` and optional `AGENTS.reference.md` ownership, provenance,
  signatures, independent `## Free Region` sections, and safe legacy migration.
- Structural and evidence conflict reporting, root fallback, shared-rule
  deduplication, and informational missing-scope suggestions.
- Compact, standard, and deep workflow budgets for local model capability
  profiles, with fixed input, output, and follow-up ceilings.

Dependency: Milestone 1 must provide reliable evidence, provenance, and
maintenance behavior first.

## Milestone Exit Criteria

- **Guidance:** Managed documents, `## Free Region`, signatures,
  provenance, validation, drift, skill workflows, and the advisory reusable
  GitHub Actions are covered by tests and locked verification passes.
- **Scoped Guidance:** Nested-document fixtures demonstrate correct scope
  discovery, inheritance, precedence, conflict reporting, deduplication,
  fallback, migration, signature configuration, and budget enforcement.

## Release-Wide Success Criteria

- Measure how many repositories generate guidance and continue using `update`
  or `audit` across later revisions.
- Track maintainer acceptance with minimal editing, evidence coverage, valid
  command and path rates, stale or unsupported rule counts, and operational
  token counts.
- Evaluate agent task success, especially fewer wrong-file, wrong-command, and
  architecture-boundary mistakes.
- Create benchmark repositories with repeatable coding tasks and judge
  generated guidance by agent behavior, not only Markdown structure.
- Keep measurement opt-in and privacy-preserving. The CLI must not require
  hosted telemetry.

## Release Boundaries

The Milestone 1 implementation is root-focused. The Rust runtime must remain
Rust only, and it must not generate semantic Markdown. Target-language fixtures
may continue to use any supported language.

Defer task-specific context packs, hosted dashboards, hosted repository
analysis, automatic local regeneration, broad team governance, enterprise
policy enforcement, additional language support without guidance-quality
improvements, automatic model or hardware detection, and static semantic
Markdown generation in Rust.

## Open Risks And Decisions

- Define the migration experience for existing documents that lack provenance
  or a `## Free Region`.
- Confirm how malformed `agentskill.toml` is surfaced to the LLM workflow while
  preserving safe default behavior.
- Keep workflow-level signature overrides temporary and ensure post-write
  validation receives the same override.
- Establish benchmark baselines for compact, standard, and deep local-model
  budget modes.
- Evaluate whether advisory drift findings are sufficient before introducing a
  strict CI mode.
