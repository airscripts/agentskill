# agentskill

[![Main](https://github.com/airscripts/agentskill/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/main.yml)
[![Release](https://github.com/airscripts/agentskill/actions/workflows/release.yml/badge.svg)](https://github.com/airscripts/agentskill/actions/workflows/release.yml)
![ClawHub](https://skill-history.com/badge/airscripts/agentskill.svg)

Analyze a code repository and synthesize an `AGENTS.md` that lets any agent
produce code consistent with the existing codebase.

<p align="center">
  <img src="https://raw.githubusercontent.com/airscripts/agentskill/main/agentskill-assets/agentskill.png" alt="agentskill" width="1280">
</p>

---

## Table of Contents

- [What It Does](#what-it-does)
- [How It Works](#how-it-works)
- [Supported Languages](#supported-languages)
- [Generation Modes](#generation-modes)
- [Installation](#installation)
- [Development Checks](#development-checks)
- [Usage](#usage)
- [Choosing a Command](#choosing-a-command)
- [References](#references)
- [Interactive Generation](#interactive-generation)
- [Update Workflow](#update-workflow)
- [Profiles and Layouts](#profiles-and-layouts)
- [Repo-Local Feedback](#repo-local-feedback)
- [Repository Layout](#repository-layout)
- [Where Code Goes](#where-code-goes)
- [Developer Workflow](#developer-workflow)
- [File Ecosystem](#file-ecosystem)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [Security](#security)
- [Releases](#releases)
- [Statistics](#statistics)
- [Support](#support)
- [License](#license)

---

## What It Does

agentskill is not a linter or a generic style-guide generator. It is a
forensic extraction tool. It walks a repository, measures source conventions,
reads formatter and linter configuration, inspects Git history, and analyzes
imports, symbols, and tests. It then emits structured evidence or a
deterministic `AGENTS.md` document.

The output is not generic advice. It is repository-specific guidance for an
agent working in an existing codebase.

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

The generation crate turns this evidence into ordered markdown sections. The
generation contract is documented in [`agentskill-skill/SYSTEM.md`](./agentskill-skill/SYSTEM.md).
The seven analyzers are implemented in Rust and run in parallel where the
workspace can safely do so.

> Read the technical background: [Turning Repository Knowledge Into Usable
> Agent Context](https://dev.to/airscript/turning-repository-knowledge-into-usable-agent-context-4pe4).

## Supported Languages

The analyzer matrix and the example fixtures cover:

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

These are target languages. agentskill itself is implemented and shipped
entirely in Rust. The fixtures under
[`agentskill-skill/examples/`](./agentskill-skill/examples/) provide compact
single-language, mixed-language, and monorepo shapes for regression coverage.

## Generation Modes

agentskill supports deterministic CLI generation and AI-assisted skill
generation.

### Static Generation (CLI)

Use the CLI when the packaged Rust runtime should produce reproducible output:

- `agentskill analyze <repo> --pretty` emits machine-readable evidence.
- `agentskill generate <repo>` creates a fresh document.
- `agentskill generate <repo> --profile comprehensive` includes richer detail.
- `agentskill generate <repo> --layout split` creates a concise document and a
  comprehensive companion.
- `agentskill generate <repo> --layout multifile` creates an index and one file
  per generated section.
- `agentskill update <repo>` regenerates sections while preserving manual text.

### AI-Assisted Generation (Skill)

The repository also contains a complete skill package in
[`agentskill-skill/`](./agentskill-skill/). An agent harness can install that
directory as a skill, run the analyzers for evidence, read `SYSTEM.md`, and
author the final `AGENTS.md` itself. This mode supports conversational
refinement and context-aware section depth.

The skill workflow uses analyzer commands as evidence gathering; it does not
need to invoke the static `generate` command. Use the CLI for deterministic
runtime output and the skill when an agent should synthesize and refine the
document interactively.

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

The skill is also listed on [ClawHub](https://clawhub.ai/airscripts/agentskill)
for harnesses that install skills from a marketplace.

## Development Checks

Install Rust through [rustup](https://rustup.rs/). The minimum supported Rust
version is 1.89. The canonical verification command is:

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

Global `--pretty` and `--out FILE` options apply to analyzer commands. `generate`
and `update` produce markdown and therefore reject `--pretty`.

```bash
# Aggregate or focused evidence
agentskill analyze <repo> --pretty
agentskill analyze <repo-a> <repo-b> --pretty
agentskill scan <repo> --pretty
agentskill measure <repo> --lang rust --pretty
agentskill config <repo> --pretty
agentskill git <repo> --pretty
agentskill graph <repo> --pretty
agentskill symbols <repo> --pretty
agentskill tests <repo> --pretty

# Save analyzer JSON
agentskill --out report.json analyze <repo>

# Generate a fresh document
agentskill generate <repo>
agentskill generate <repo> --out AGENTS.md
agentskill generate <repo> --profile comprehensive
agentskill generate <repo> --layout split
agentskill generate <repo> --layout multifile

# Update an existing document
agentskill update <repo>
agentskill update <repo> --section testing
agentskill update <repo> --exclude-section git
agentskill update <repo> --force
agentskill update <repo> --out updated-AGENTS.md
```

Use `agsk` in place of `agentskill` for every command. Run
`agentskill --help` or `agentskill <command> --help` for the exact current
Clap syntax.

## Choosing a Command

Use `analyze` when you want JSON from all analyzers without writing markdown.
It accepts one or more repositories and is the contract-stable inspection
path. Use an individual analyzer when a focused signal is needed.

Use `generate` for a new document. Single-layout generation prints markdown to
stdout by default and only writes a file when `--out` is supplied. It never
merges with an existing `AGENTS.md`.

Use `update` when an `AGENTS.md` already exists or when you want deterministic
regeneration with preservation. It writes to `<repo>/AGENTS.md` by default, or
to `--out` while still using the repository's existing document as merge input.

## References

`analyze` and `generate` accept repeatable `--reference` flags. A local
reference must be a directory containing a readable, non-empty `AGENTS.md`.
Remote Git URLs (`http://`, `https://`, `ssh://`, or `git@...`) are cloned shallowly and
read the same way. References are explicit inputs, are validated before use,
and duplicate local sources are rejected.

```bash
agentskill analyze <repo> --reference ../reference-repo --pretty
agentskill generate <repo> \
  --reference ../reference-a \
  --reference https://github.com/example/reference.git
```

Reference provenance is retained in generated metadata, including source and
commit information when available. References do not silently change the
analyzer JSON contract.

## Interactive Generation

`generate --interactive` is opt-in gap filling. It asks when important signals
are unavailable, such as a canonical test command or Git conventions. Answers
are inserted as explicit notes in the relevant sections;
an answer inferred from a supplied reference avoids an unnecessary prompt.

```bash
agentskill generate <repo> --interactive
```

Review generated notes against the repository when evidence from multiple
sources differs.

## Update Workflow

`agentskill update <repo>` analyzes the repository, regenerates generated
sections, merges them with the existing document, and writes the result back.

- `--section NAME` regenerates only named sections.
- `--exclude-section NAME` leaves named generated sections untouched.
- Missing targeted sections are inserted without rewriting unrelated manual
  sections.
- Untouched custom sections and preamble text remain in place in normal mode.
- `--force` performs a clean-slate rebuild and ignores preservation hints.

`update` currently supports only the default `single` layout. Passing `split`
or `multifile` is rejected clearly.

## Profiles and Layouts

### Profiles (content density)

`generate` and `update` accept `--profile`:

- `concise` (default) contains operational rules and key facts.
- `comprehensive` adds a verification reminder to each generated section for
  workflows that need more guidance while reviewing evidence.

Both profiles are deterministic and preserve section headings and order.

### Layouts (Output Packaging)

`generate` accepts `--layout`:

- `single` (default) emits one complete markdown document.
- `split` writes a concise `AGENTS.md` and an `AGENTS.reference.md` companion;
  the primary links to the companion. The primary is always concise and the
  companion always comprehensive.
- `multifile` writes a compact `AGENTS.md` index and section files in a
  `.agentskill/` directory. Section filenames use stable numbering, for
  example `01_OVERVIEW.md`, `05_COMMANDS_AND_WORKFLOWS.md`, and
  `12_TESTING.md`:

  ```text
  .agentskill/
    01_OVERVIEW.md
    02_REPOSITORY_STRUCTURE.md
    05_COMMANDS_AND_WORKFLOWS.md
    06_CODE_FORMATTING.md
    07_NAMING_CONVENTIONS.md
    08_TYPE_ANNOTATIONS.md
    09_IMPORTS.md
    10_ERROR_HANDLING.md
    11_COMMENTS_AND_DOCSTRINGS.md
    12_TESTING.md
    13_GIT.md
    14_DEPENDENCIES_AND_TOOLING.md
    15_RED_LINES.md
  ```

When `--out` is omitted, split and multifile write into the target repository.
For single layout, markdown goes to stdout unless `--out` is supplied.

| Layout | Profile behavior | Default profile |
| --- | --- | --- |
| `single` | Controls the one output document | `concise` |
| `split` | Ignored; primary is concise and companion comprehensive | N/A |
| `multifile` | Controls each section file | `concise` |

## Repo-Local Feedback

Incremental updates can read an optional, version-controlled
`.agentskill-feedback.json` beside the repository's `AGENTS.md`:

```json
{
  "sections": {
    "overview": {
      "prepend_notes": ["Deployments go through GitHub Actions."]
    },
    "testing": {
      "pinned_facts": ["Use cargo test as the canonical test runner."]
    }
  },
  "preserve_sections": ["red lines"]
}
```

Supported keys are intentionally narrow: `sections.<name>.prepend_notes`,
`sections.<name>.pinned_facts`, and `preserve_sections`. In normal update mode,
preserved sections act like an implicit exclusion list. `--force` ignores those
hints. Use the sidecar for durable regeneration guidance; edit `AGENTS.md`
directly for one-off manual text.

## Repository Layout

```text
README.md                 # user-facing overview and contributor workflow
AGENTS.md                 # conventions for this repository itself
Cargo.toml                # Rust workspace definition
Cargo.lock                # reproducible dependency resolution
agentskill-core/          # shared types, filesystem, language registry
agentskill-analyzers/     # seven analyzers and aggregate execution
agentskill-generation/    # rendering, references, layouts, and merging
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
- Put document rendering, profiles, layouts, references, feedback, and update
  merging in `agentskill-generation/`.
- Keep `agentskill/src/main.rs` thin; route CLI behavior through the library
  crates and expose both binaries from `agentskill/`.
- Keep `agentskill-scripts/` limited to release, archive, and operator helpers;
  do not put analyzer or generation logic there.
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
languages, and generation/update semantics are compatibility surfaces.

## File Ecosystem

Read these files together before changing generation behavior:

| File | Role |
| --- | --- |
| [`agentskill-skill/SYSTEM.md`](./agentskill-skill/SYSTEM.md) | Contract for generated `AGENTS.md` sections |
| [`agentskill-skill/SKILL.md`](./agentskill-skill/SKILL.md) | AI-assisted evidence and synthesis workflow |
| [`agentskill-skill/references/GOTCHAS.md`](./agentskill-skill/references/GOTCHAS.md) | Extraction and synthesis errors to avoid |
| [`agentskill-docs/cli.md`](./agentskill-docs/cli.md) | Detailed CLI surface |
| [`agentskill-docs/architecture.md`](./agentskill-docs/architecture.md) | Crate boundaries and data flow |
| [`CONTRIBUTING.md`](./CONTRIBUTING.md) | Contributor and release expectations |

## Examples

[`agentskill-skill/examples/README.md`](./agentskill-skill/examples/README.md)
indexes compact fixtures for every supported target language and reference
outputs for single-language, multi-language, and monorepo repositories. They
are used by analyzer coverage and contract tests, and are useful when checking
how language detection or test mapping behaves.

Try one locally:

```bash
agentskill analyze agentskill-skill/examples/python --pretty
agentskill scan agentskill-skill/examples/typescript --pretty
agentskill generate agentskill-skill/examples/mixed
```

## API Reference

Contributor-oriented documentation lives under
[`agentskill-docs/`](./agentskill-docs/):

- [`cli.md`](./agentskill-docs/cli.md) describes commands, flags, and output.
- [`architecture.md`](./agentskill-docs/architecture.md) describes crate
  responsibilities, analyzer contracts, generation, and release flow.

The Rust crates are the implementation source of truth; the docs summarize
their public boundaries without exposing every private helper.

## Contributing

Contributions are welcome, especially improvements to analyzer depth,
deterministic generation, supported-language fixtures, compatibility contracts,
and skill ergonomics. Before opening a pull request, read
[`CONTRIBUTING.md`](./CONTRIBUTING.md) and
[`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md). Use the repository issue and pull
request templates when reporting bugs or proposing changes.

## Security

See [`SECURITY.md`](./SECURITY.md) for supported versions and vulnerability
reporting guidance. Dependency policy is checked with `cargo deny` and the
release workflow validates archives before publishing them.

## Statistics

Track the project's public star history:

[![Star History Chart](https://api.star-history.com/chart?repos=airscripts/agentskill&type=date&legend=top-left)](https://www.star-history.com/?repos=airscripts%2Fagentskill&type=date&legend=top-left)

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
