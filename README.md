# Agentskill

[![Main](https://github.com/airscripts/agentskill/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/main.yml)
[![Release](https://github.com/airscripts/agentskill/actions/workflows/release.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/release.yml)

Analyze a repository and synthesize an `AGENTS.md` that lets an agent produce
code consistent with the existing codebase.

## What It Does

agentskill walks a repository, measures source conventions, reads tool
configuration, inspects Git history, and emits structured evidence or a
deterministic `AGENTS.md` document.

The analyzer matrix covers Python, TypeScript, JavaScript, Go, Rust, Java,
Kotlin, C#, C, C++, Ruby, PHP, Swift, Objective-C, and Bash. These are target
languages; agentskill itself is implemented and shipped entirely in Rust.

## Installation

Download the archive for your platform from
[GitHub Releases](https://github.com/airscripts/agentskill/releases), extract
it, and place either `agentskill` or `agsk` on your `PATH`. Verify downloads
with the published `SHA256SUMS` file.

For development from source:

```bash
cargo install --git https://github.com/airscripts/agentskill agentskill
```

## Usage

```bash
agentskill analyze <repo> --pretty
agentskill scan <repo> --pretty
agentskill measure <repo> --lang rust --pretty
agentskill config <repo> --pretty
agentskill git <repo> --pretty
agentskill graph <repo> --pretty
agentskill symbols <repo> --pretty
agentskill tests <repo> --pretty

agentskill generate <repo>
agentskill generate <repo> --out AGENTS.md
agentskill generate <repo> --profile comprehensive
agentskill generate <repo> --layout split
agentskill generate <repo> --layout multifile

agentskill update <repo>
agentskill update <repo> --section testing
agentskill update <repo> --exclude-section git
agentskill update <repo> --force
```

`agsk` is an equivalent short executable name. Analyzer commands emit JSON;
`--pretty` formats it and `--out FILE` writes it to disk. `generate` creates a
fresh document, while `update` preserves custom sections and supports targeted
regeneration. References are supplied with repeatable `--reference` flags.

## Development

Required local checks:

```bash
make verify
```

Individual targets include `make build`, `make coverage`, `make security`,
`make workflows`, and `make fmt`.

The workspace uses Rust 1.89 as its minimum supported toolchain. The root
`Cargo.toml` defines the crate boundaries and `Cargo.lock` is committed for
reproducible builds.

## Repository Layout

```text
agentskill-core/        Shared types, errors, language registry, and documents
agentskill-analyzers/   Repository analyzers and aggregate execution
agentskill-generation/  AGENTS.md rendering, references, and update merging
agentskill/              Clap CLI with agentskill and agsk binaries
agentskill-skill/        Skill instructions, references, and target fixtures
agentskill-scripts/      Release-note and archive validation helpers
agentskill-docs/         CLI and architecture reference
agentskill-assets/       Repository artwork
agentskill-tests/        Contract fixtures for compatibility checks
```

The packaged skill entrypoint is `agentskill-skill/SKILL.md` and its generation
contract is `agentskill-skill/SYSTEM.md`. `CHANGELOG.md` is the source for final
GitHub Release notes.

For the technical implementation map, see
[`agentskill-docs/architecture.md`](./agentskill-docs/architecture.md).

## Releases

Releases are tag-driven and fully automated through GitHub Actions. Numeric
tags use `X.Y.Z` for stable releases and `X.Y.Z-rc.N` for prereleases. The
workflow validates the tag against `VERSION`, runs verification and the full
test matrix, builds six platform archives containing both binaries, creates
SHA256 checksums, and publishes the GitHub Release. Stable release notes are
extracted from the matching `CHANGELOG.md` section.

## License

MIT. See [LICENSE](./LICENSE).
