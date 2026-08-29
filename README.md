# Agentskill

[![Main](https://github.com/airscripts/agentskill/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/main.yml)
[![Release](https://github.com/airscripts/agentskill/actions/workflows/release.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/release.yml)

Turn any codebase into an `AGENTS.md` playbook AI coding agents can actually
follow.

<p align="center">
  <img src="https://raw.githubusercontent.com/airscripts/agentskill/main/agentskill-assets/agentskill.png" alt="agentskill" width="1280">
</p>

---

## Table Of Contents

- [What It Does](#what-it-does)
- [How It Works](#how-it-works)
- [Supported Languages](#supported-languages)
- [Generation Modes](#generation-modes)
- [Installation](#installation)
- [Development Checks](#development-checks)
- [Usage](#usage)
- [Choosing a Command](#choosing-a-command)
- [AGENTS.md Output](#agentsmd-output)
- [Maintainer Context](#maintainer-context)
- [Repository Layout](#repository-layout)
- [Where Code Goes](#where-code-goes)
- [Developer Workflow](#developer-workflow)
- [File Ecosystem](#file-ecosystem)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [Security](#security)
- [Releases](#releases)
- [Support](#support)
- [License](#license)

---

## What It Does

agentskill is not a linter or a generic style-guide generator. It is a
forensic extraction tool. It walks a repository, measures source conventions,
reads formatter and linter configuration, inspects Git history, and analyzes
imports, symbols, and tests. It then emits structured evidence for an LLM.

The LLM skill uses that evidence to author compact, repository-specific
`AGENTS.md` guidance.

## How It Works

Seven analyzers run independently and their results are combined in a stable
JSON shape:

| Analyzer | What it measures |
| --- | --- |
| `scan` | Directory tree, file inventory, languages, and suggested read order |
| `measure` | Indentation, line-length percentiles, blank lines, and whitespace |
| `config` | Formatter, linter, type-checker, editor, and project configuration |
| `git` | Commit subjects, prefixes, branches, merge signals, and history |
| `graph` | Internal imports, cycles, dependency concentration, and boundaries |
| `symbols` | Functions, types, constants, naming patterns, and affixes |
| `tests` | Test frameworks, mappings, fixtures, and test commands |

The evidence contract and LLM output rules are documented in
[`agentskill-skill/SYSTEM.md`](./agentskill-skill/SYSTEM.md).
The seven analyzers are implemented in Rust and run in parallel where the
workspace can safely do so.

> Read the technical background: [Turning Repository Knowledge Into Usable
> Agent Context](https://dev.to/airscript/turning-repository-knowledge-into-usable-agent-context-4pe4).

## Supported Languages

The analyzer matrix and the example fixtures cover 60 language families:

- Python
- TypeScript
- JavaScript
- Go
- Rust
- Java
- Kotlin
- C#
- C
- C++
- Ruby
- PHP
- Swift
- Objective-C
- Shell / Bash
- Dart
- Scala
- Elixir
- Erlang
- Lua
- R
- Julia
- Haskell
- Clojure
- F#
- Groovy
- PowerShell
- Visual Basic .NET
- Zig
- D
- Nim
- Crystal
- OCaml
- Perl
- MATLAB
- Fortran
- Ada
- GDScript
- Solidity
- HTML
- Vue
- Svelte
- Astro
- CSS
- Sass / SCSS
- Less
- SQL
- GraphQL
- Protocol Buffers
- HCL / Terraform
- Nix
- Dockerfile
- Make
- CMake
- Starlark

YAML, JSON, TOML, XML, and Markdown are detected as auxiliary formats. They
appear under the analyzer `auxiliary` object and are excluded from dominant
language summaries and generated language guidance.

The `.m` extension is ambiguous between MATLAB and Objective-C. Content and
repository markers are used when available; otherwise static detection favors
Objective-C. Use `--lang matlab` when analyzing marker-free MATLAB files.

These are target languages. agentskill itself is implemented and shipped
entirely in Rust. The fixtures under
[`agentskill-skill/examples/`](./agentskill-skill/examples/) provide compact
single-language, mixed-language, and monorepo shapes for regression coverage.

## Generation Modes

The repository also contains a complete skill package in
[`agentskill-skill/`](./agentskill-skill/). An agent harness can install that
directory as a skill, run the evidence command, read `SYSTEM.md`, and author
the final `AGENTS.md` itself. Semantic Markdown generation happens through the
LLM skill rather than the Rust CLI.

The skill supports `init`, `enrich`, `scope`, `context`, `update`, and `audit`
workflows. `operational` output is the compact root document; `reference`
output is deeper context loaded only when needed.

## Installation

Download the archive for your platform from
[GitHub Releases](https://github.com/airscripts/agentskill/releases), extract
it, and put either `agentskill` or `agsk` on your `PATH`. Verify downloads with
the release's `SHA256SUMS` file.

For a source checkout, install the release binary with Cargo:

```bash
cargo install --git https://github.com/airscripts/agentskill agentskill
```

Both binary names are built from the workspace. `agsk` is an equivalent short
name for `agentskill`.

### For Agents

Install the repository root as a skill when your harness supports filesystem or
Git skill installation. The relevant package layout is:

```text
agentskill-skill/
  SKILL.md            # skill entrypoint and workflow
  SYSTEM.md           # generated-document contract
  references/         # extraction and synthesis guidance
  examples/           # target-language fixtures and reference shapes
```

If the harness only needs the analyzer runtime, install the binaries and use
the commands below. The skill package and the Rust CLI are intentionally
separate: the former gives an agent a synthesis workflow, while the latter
provides deterministic evidence and document operations.

## Development Checks

Install Rust through [rustup](https://rustup.rs/) and Lefthook with
`cargo install lefthook`, then enable the repository hooks:

```bash
lefthook install
```

The minimum supported Rust version is 1.89. The canonical verification command
is:

```bash
make verify
```

This runs locked linting and compilation, the complete workspace test suite,
and workflow/script validation. Individual targets are
available when iterating:

```bash
make build       # release binaries
make check       # cargo check --workspace --locked
make fmt         # cargo fmt --all
make lint        # clippy with -D warnings
make test        # cargo test --workspace --locked
make coverage    # llvm-cov with the 80% line threshold
make security    # cargo-deny dependency policy checks
make workflows   # actionlint and shellcheck
```

`Cargo.lock` is committed so local and CI builds use reproducible dependency
resolution. Optional staged-file checks are configured through `lefthook.yml`
and `agentskill-scripts/pre-commit.sh`.

## Usage

Global `--pretty` and `--out FILE` options apply to static JSON commands. The
CLI never writes semantic Markdown.

```bash
# Aggregate or focused evidence.
agentskill analyze <repo> --pretty
agentskill analyze <repo-a> <repo-b> --pretty
agentskill evidence <repo> --pretty
agentskill scan <repo> --pretty
agentskill measure <repo> --lang rust --pretty
agentskill config <repo> --pretty
agentskill git <repo> --pretty
agentskill graph <repo> --pretty
agentskill symbols <repo> --pretty
agentskill tests <repo> --pretty

# Save analyzer JSON.
agentskill --out report.json analyze <repo>

# Validate and inspect LLM-authored documents.
agentskill validate <repo>
agentskill drift <repo>
```

Use `agsk` in place of `agentskill` for every command. Run
`agentskill --help` or `agentskill <command> --help` for the exact current
Clap syntax.

## Choosing A Command

Use `analyze` when you want JSON from all analyzers without writing markdown.
It accepts one or more repositories and is the contract-stable inspection
path. Use an individual analyzer when a focused signal is needed.

Use `evidence` when an LLM needs normalized facts with scope, confidence, and
provenance. Use an individual analyzer when a focused static signal is needed.

Use the installed skill for `init`, `enrich`, `scope`, `context`, `update`, and
`audit`. Use `validate` and `drift` after the skill writes or updates documents.

## AGENTS.md Output

The skill writes two optional depth views from the same evidence:

- `AGENTS.md` is the compact operational contract. Keep it self-sufficient,
  imperative, and normally within 500–1,000 tokens; treat 1,500 as a hard
  ceiling.
- `AGENTS.reference.md` is unrestricted supporting context: architecture,
  rationale, evidence details, workflows, and examples that an agent can load
  selectively. The root file links to it when it exists.

The root document should prioritize repository mission and map, non-negotiable
rules, conceptual don'ts, quick-start commands, change routing, architecture,
testing, and only the most useful playbooks. It should state verified facts as
rules, avoid raw analyzer dumps, and omit uncertain conventions. The reference
document can preserve depth without spending every agent's context window.

The LLM skill is responsible for `init`, `enrich`, `scope`, `context`, `update`,
and `audit`. The CLI remains deterministic and read-only: `evidence` supplies
facts, while `validate` and `drift` check the documents after the skill writes
them.

## Maintainer Context

Durable maintainer answers belong in the repository's normal review flow or in
the reference document, not in a CLI feedback sidecar. During generation the
skill should ask only high-impact questions that static evidence cannot answer,
record the resulting decision in the appropriate document, and distinguish
maintainer policy from repository observation.

## Repository Layout

```text
README.md                 # user-facing overview and contributor workflow
AGENTS.md                 # conventions for this repository itself
Cargo.toml                # Rust workspace definition
Cargo.lock                # reproducible dependency resolution
agentskill-core/          # shared types, filesystem, language registry
agentskill-analyzers/     # seven analyzers and aggregate execution
agentskill-generation/    # validation and evidence/document drift checks
agentskill/               # Clap CLI and agentskill/agsk binaries
agentskill-skill/         # skill instructions, references, and fixtures
agentskill-scripts/       # release and archive verification helpers
agentskill-docs/          # CLI and architecture references
agentskill-assets/        # repository artwork
agentskill-tests/         # compatibility contract fixtures
.github/                  # CI, release workflows, and issue templates
```

## Where Code Goes

- Put shared domain types, filesystem behavior, errors, and language detection
  in `agentskill-core/`.
- Put analyzer implementations and aggregate execution in
  `agentskill-analyzers/`.
- Put document validation and evidence/document drift checks in the
  `agentskill-generation/` package (published as `agentskill-validation`).
- Keep `agentskill/src/main.rs` thin; route CLI behavior through the library
  crates and expose both binaries from `agentskill/`.
- Keep `agentskill-scripts/` limited to release, archive, and operator helpers;
  do not put analyzer or validation logic there.
- Keep target-language fixtures under `agentskill-skill/examples/` and contract
  fixtures under `agentskill-tests/`.

Do not reintroduce Python runtime code, package setup, or Python CI workflows.
Python fixtures remain supported because Python is one of the analyzed target
languages.

## Developer Workflow

For a normal change:

1. Read `AGENTS.md`, the owning crate, and the relevant contract tests.
2. Keep public behavior deterministic: stable section ordering, sorted paths,
   and reproducible JSON values.
3. Add unit or integration coverage in the owning crate.
4. Update user-facing docs and `CHANGELOG.md` when a public command, flag,
   output key, or generated-document behavior changes.
5. Run `make fmt`, then `make verify` before opening a pull request.

Public command names, flags, analyzer keys, error payloads, supported target
languages, evidence fields, and document validation semantics are compatibility
surfaces.

## File Ecosystem

Read these files together before changing evidence or document behavior:

| File | Role |
| --- | --- |
| [`agentskill-skill/SYSTEM.md`](./agentskill-skill/SYSTEM.md) | Contract for LLM-authored `AGENTS.md` files |
| [`agentskill-skill/SKILL.md`](./agentskill-skill/SKILL.md) | AI-assisted evidence and synthesis workflow |
| [`agentskill-skill/references/GOTCHAS.md`](./agentskill-skill/references/GOTCHAS.md) | Extraction and synthesis errors to avoid |
| [`agentskill-docs/cli.md`](./agentskill-docs/cli.md) | Detailed CLI surface |
| [`agentskill-docs/architecture.md`](./agentskill-docs/architecture.md) | Crate boundaries and data flow |
| [`CONTRIBUTING.md`](./CONTRIBUTING.md) | Contributor and release expectations |

## Examples

[`agentskill-skill/examples/README.md`](./agentskill-skill/examples/README.md)
indexes compact fixtures for every supported target language and reference
context examples for single-language, multi-language, and monorepo repositories.
They are used by analyzer coverage and contract tests, and are useful when
checking how language detection or test mapping behaves.

Try one locally:

```bash
agentskill analyze agentskill-skill/examples/python --pretty
agentskill scan agentskill-skill/examples/typescript --pretty
agentskill evidence agentskill-skill/examples/mixed --pretty
```

## API Reference

Contributor-oriented documentation lives under
[`agentskill-docs/`](./agentskill-docs/):

- [`cli.md`](./agentskill-docs/cli.md) describes commands, flags, and output.
- [`architecture.md`](./agentskill-docs/architecture.md) describes crate
  responsibilities, evidence contracts, validation, and release flow.

The Rust crates are the implementation source of truth; the docs summarize
their public boundaries without exposing every private helper.

## Contributing

Contributions are welcome, especially improvements to analyzer depth,
evidence quality, supported-language fixtures, compatibility contracts, and
skill ergonomics. Before opening a pull request, read
[`CONTRIBUTING.md`](./CONTRIBUTING.md) and
[`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md). Use the repository issue and pull
request templates when reporting bugs or proposing changes.

## Security

See [`SECURITY.md`](./SECURITY.md) for supported versions and vulnerability
reporting guidance. Dependency policy is checked with `cargo deny` and the
release workflow validates archives before publishing them.

## Releases

Releases are tag-driven and automated through GitHub Actions. Stable tags use
`X.Y.Z`; prereleases use `X.Y.Z-rc.N`. The workflow validates the tag against
`VERSION`, extracts stable notes from the matching `CHANGELOG.md` section, runs
locked verification and the full test matrix, builds six platform archives
containing both binaries plus `LICENSE`, generates `SHA256SUMS`, and publishes
the GitHub Release.

## Support

- [GitHub Sponsors](https://github.com/sponsors/airscripts)
- [Ko-fi](https://ko-fi.com/airscript)

Bug reports and feature requests belong in the repository's issue tracker.
Starring, sharing, contributing fixes, and supporting the project all help.

## License

MIT. See [`LICENSE`](./LICENSE).
