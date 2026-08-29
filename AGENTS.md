# AGENTS.md

## Mission

agentskill turns repositories into compact, evidence-backed `AGENTS.md`
playbooks that AI coding agents can actually follow. It is a Rust CLI shipped
as `agentskill` and `agsk`; the packaged LLM skill authors semantic Markdown.

## Repository Map

- `agentskill-core/`: shared errors, language registry, filesystem, roles, and
  document parsing.
- `agentskill-analyzers/`: scan, measure, config, Git, graph, symbols, tests,
  and normalized evidence.
- `agentskill-generation/`: validation package (`agentskill-validation`) for
  read-only document checks and drift reporting.
- `agentskill/`: thin Clap CLI and both binaries.
- `agentskill-skill/`: LLM workflow, output contract, references, and fixtures.
- `agentskill-docs/`: CLI and architecture documentation.

## Non-Negotiables

- Keep the agentskill runtime Rust-only; target-language fixtures may use any
  supported language, including Python.
- Keep logic in its owning crate and keep `agentskill/src/main.rs` thin.
- Preserve deterministic output: sorted paths, stable keys, bounded reads, and
  reproducible analyzer results.
- Treat analyzer keys, evidence fields, CLI flags, supported languages, and
  validation behavior as compatibility surfaces; update tests and docs with
  intentional changes.

## Don'ts

- Do not generate or update semantic Markdown in Rust; the LLM skill owns
  document authorship.
- Do not put analyzer, evidence, or validation logic in the CLI entrypoint.
- Do not treat target-language fixtures as production code or remove supported
  languages and their example fixtures.
- Do not turn counts, heuristics, or low-confidence evidence into repository
  rules.
- Do not bypass locked verification, release archive checks, or checksum
  generation.

## Quick Start

```bash
make fmt
make lint
make check
make test
make verify
```

Use `cargo run --bin agentskill -- <command> ...` or the equivalent `agsk`
binary while iterating. Run `agentskill evidence <repo> --pretty` for the
LLM input, then `agentskill validate <repo>` and `agentskill drift <repo>`
after writing documents.

## Change Routing

Put shared behavior in `agentskill-core/`, analyzer behavior in
`agentskill-analyzers/`, document checks in `agentskill-generation/`, and CLI
dispatch only in `agentskill/`. Update `agentskill-skill/SYSTEM.md` and
`agentskill-skill/SKILL.md` when the LLM document contract changes. Update user-facing docs and
`CHANGELOG.md` for public behavior changes.

## Testing And Release

Run the owning crate tests while iterating, then the locked workspace checks.
Release tags use `X.Y.Z` or `X.Y.Z-rc.N`; `VERSION` matches the workspace
version. Archives must contain both binaries and `LICENSE`, plus `SHA256SUMS`.

For deeper architecture, evidence semantics, workflows, and rationale, read
[`AGENTS.reference.md`](AGENTS.reference.md) selectively.
