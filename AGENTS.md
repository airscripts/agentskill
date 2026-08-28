# AGENTS.md

## Overview

agentskill is a Rust CLI distributed as `agentskill` and `agsk` binaries. It
analyzes one or more repositories, emits structured analyzer output, and
supports deterministic `AGENTS.md` generation and in-place updates.

## Repository Structure

```text
agentskill-core/        Shared errors, types, filesystem, language registry
agentskill-analyzers/   Seven analyzers and aggregate execution
agentskill-generation/  References, rendering, layouts, and update merging
agentskill/              Clap CLI and binary targets
agentskill-skill/        Skill instructions, references, and target fixtures
agentskill-scripts/      Release-note and archive verification helpers
agentskill-docs/         CLI and architecture reference
agentskill-assets/       Repository artwork
agentskill-tests/        Contract fixtures for compatibility checks
```

Keep implementation logic in its owning crate. Keep `agentskill/src/main.rs`
thin and route behavior through the library crates. Target-language fixtures
may use Python or any other supported language; the agentskill implementation
must remain Rust-only.

## Commands And Workflows

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --locked
cargo test --workspace --locked
```

Run the CLI locally with `cargo run --bin agentskill -- <command> ...` or
`cargo run --bin agsk -- <command> ...`. Release artifacts are built and
published by GitHub Actions from numeric `X.Y.Z` and `X.Y.Z-rc.N` tags.

## Rust Conventions

- Use Rust 2024 edition and preserve the MSRV of 1.89.
- Run rustfmt; do not hand-format around it.
- Keep public APIs documented when they cross crate boundaries.
- Prefer typed domain structs at crate boundaries and `serde_json::Value` only
  for the intentionally JSON-shaped analyzer contract.
- Return tolerant per-file or per-analyzer errors where a repository scan can
  continue; reserve process failure for invalid CLI arguments or unusable paths.
- Keep output deterministic: stable section ordering, sorted file paths, and
  reproducible JSON values.

## Testing

Tests live in each crate's `tests/` directory or in source modules for focused
unit behavior. Cover command flags, exact analyzer keys and error payloads,
language detection, references, generation profiles/layouts, document merging,
both binary names, and release helper scripts.

## Release Rules

`VERSION` is the stable base version and must match the workspace version.
Final release notes are extracted from the matching `CHANGELOG.md` section.
RC tags publish prereleases with generated candidate notes. Archives must
contain `agentskill`, `agsk`, and `LICENSE`, and the release must include
`SHA256SUMS`.

## Red Lines

- Do not reintroduce Python runtime code, package setup, or Python CI workflows.
- Do not remove supported target languages or their example fixtures.
- Do not place analyzer or generation logic in the CLI entrypoint.
- Do not change stable JSON/markdown behavior without updating contract tests,
  documentation, and the changelog.
- Do not publish an artifact without locked verification and archive checks.
