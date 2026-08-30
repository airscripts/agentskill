# Agentskill Reference Context

This document is the deep context behind the compact root contract. Load only
the sections relevant to the change. It is intentionally allowed to be much
larger than `AGENTS.md`; do not duplicate all of it in the operational file.

## Product Boundary

agentskill is both a Rust repository-analysis CLI and a packaged LLM skill. The
Rust side answers questions that can be verified mechanically: what files and
languages exist, how code is structured, which tools and commands are
configured, how tests are organized, and what Git history suggests. The LLM
skill combines that evidence with direct source inspection and explicit
maintainer answers to author `AGENTS.md`.

The CLI does not generate semantic Markdown. `evidence` is the synthesis input;
`validate` and `drift` are read-only post-write checks. The skill owns `init`,
`enrich`, `scope`, `context`, `update`, and `audit` workflows. `AGENTS.md` is
the operational depth view; `AGENTS.reference.md` is optional deep context.

## Workspace Ownership

```text
agentskill-core/
  errors, language registry, bounded filesystem walk, file roles,
  document parsing and stable JSON output
agentskill-analyzers/
  scan, measure, config, git, graph, symbols, tests, aggregate execution,
  repository snapshots, and normalized evidence facts
agentskill-generation/
  source directory for the agentskill-validation package; document validation
  and broken-reference checks only
agentskill/
  Clap parsing, command dispatch, output and process exit status,
  the agentskill and agsk binaries
agentskill-skill/
  LLM workflow instructions, output contract, synthesis gotchas, and context
  examples
agentskill-docs/
  detailed CLI and architecture documentation
agentskill-scripts/
  release-note extraction, archive verification, and filtered pre-commit checks
agentskill-tests/
  analyzer compatibility fixtures, guidance fixtures, and contract schemas
```

Keep dependencies pointed toward stable lower layers. `agentskill/src/main.rs`
should remain a binary shim; command behavior belongs in the CLI library and
analysis belongs in analyzer modules. The source directory
`agentskill-generation/` is retained for layout compatibility, but its package
name is `agentskill-validation`.

## Evidence Contract

`agentskill evidence <repo> --pretty` emits schema version `3` with four useful
parts:

```json
{
  "schema_version": 3,
  "repository": {
    "root": "...",
    "revision": "...",
    "dirty": false
  },
  "facts": [
    {
      "id": "test.command.1",
      "category": "command",
      "scope": "repository",
      "value": "cargo test --workspace --locked",
      "confidence": "verified",
      "evidence": [{"path": "Makefile"}]
    }
  ],
  "analyzers": {}
}
```

Facts are compact navigation and synthesis material. Each has an identifier,
category, scope, value, confidence, and provenance. Do not turn raw counts or
heuristic guesses into imperative rules. Verify important facts in source,
configuration, CI, tests, or Git history before writing them into the root
document.

The aggregate analyzer snapshot is bounded and deterministic. The filesystem
walk sorts entries, avoids symlinks, skips dependency/cache/build trees, reads
at most one megabyte per file, and caps parsed files. It classifies recognized
files as source, test, example, documentation, generated, configuration, or
auxiliary. Hidden repository directories such as `.github/` are included so CI
and workflow evidence is available.

The raw analyzer object contains these stable keys:

- `scan`: tree, inventory, language totals, and suggested read order.
- `measure`: indentation, line-length, blank-line, newline, and whitespace
  observations.
- `config`: formatter, linter, type-checker, project-marker, and editor data.
- `git`: commit prefixes, subjects, branches, merge signals, and history.
- `graph`: internal imports, cycles, dependency concentration, and boundaries.
- `symbols`: declarations and naming-pattern extraction.
- `tests`: frameworks, source/test mapping, fixtures, naming patterns, and
  command inference.

The current repository evidence strongly supports Rust as the implementation
language and Bash for repository scripts. It also contains many target-language
fixtures. Do not treat those fixtures as production implementations or infer a
multi-language runtime from their presence.

## Operational Document Rules

The root document should be self-sufficient and normally fit within 500–1,000
tokens. Treat 1,500 tokens as a hard ceiling. Prefer imperative rules that
change agent behavior over descriptive inventory. The intended order is:

1. mission and repository map;
2. non-negotiables;
3. three to seven conceptual don'ts;
4. quick-start commands;
5. change routing;
6. architecture and implementation conventions;
7. testing and validation;
8. a few high-value change playbooks; and
9. a link to further context.

Include a rule only when it is backed by source/configuration evidence or an
explicit maintainer decision. Scope language-specific rules to the relevant
crate, path, service, or fixture. Avoid pasting analyzer JSON, transient Git
revisions, unsupported tools, or rationale that belongs in this file.

If `AGENTS.reference.md` exists, keep the link in the root document. The root
must still be useful when an agent does not load the reference file.

## Reference Document Rules

The reference file has no compact token budget. Use it for details that improve
accuracy but are not needed on every coding turn: crate ownership, evidence
interpretation, architecture rationale, source-backed examples, test topology,
release mechanics, known uncertainty, and historical decisions. It is valid to
grow this file as the product learns more about the repository.

For very large repositories, split deep context into topic files and keep the
root operational file as the stable index. Load only the topic needed for the
current task. Never let reference depth turn uncertain observations into rules.

## Commands And Verification

The repository's canonical local checks are:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --locked
cargo test --workspace --locked
```

The `Makefile` wraps these as `make fmt`, `make lint`, `make check`, and
`make test`; `make verify` additionally runs workflow and shell validation.
Use `cargo run --bin agentskill -- <command> ...` or
`cargo run --bin agsk -- <command> ...` to iterate locally.

The relevant CLI surface is:

```text
analyze <repo>...
evidence <repo>
scan <repo>
measure <repo>
config <repo>
git <repo>
graph <repo>
symbols <repo>
tests <repo>
validate <repo>
drift <repo>
```

All JSON commands support `--pretty` and `--out FILE`; language-aware commands
support `--lang`. Analyzer failures are structured JSON with a failed process
status. `validate` checks document existence, normalized duplicate headings,
local references, trailing newline, and operational token budget. `drift`
reruns evidence, reports broken references in both documents, and exposes the
current repository revision. Both commands are read-only.

## Testing Topology

Focused unit behavior lives beside implementation or in each crate's test
directory. `agentskill-analyzers/tests/contracts.rs` protects aggregate keys,
error shapes, language filtering, and evidence provenance. The analyzer
coverage suite exercises the supported language matrix and edge cases.

`agentskill-core/tests/core.rs` protects language detection, filesystem safety,
file-role classification, and document parsing. The validation workflow tests
protect duplicate-heading detection, broken references, read-only drift, and
operational/reference document handling. CLI tests cover both binary names and
JSON dispatch.

When changing public JSON, language detection, file roles, command flags, or
validation semantics, update the owning contract tests and documentation in
the same change. Keep target-language fixtures under
`agentskill-skill/examples/`; they may contain Python or other languages, while
the agentskill implementation remains Rust-only.

## Git And Release Signals

Repository history uses conventional prefixes such as `feat`, `fix`, `docs`,
`refactor`, `test`, `build`, `ci`, `chore`, `deps`, `style`, and `release`.
Use a conventional prefix for commits unless a maintainer says otherwise.
The analyzer does not have enough evidence to assert a merge strategy.

`VERSION` and the workspace version are `2.1.0`. Stable releases use numeric
`X.Y.Z` tags; release candidates use `X.Y.Z-rc.N`. Release notes come from the
matching `CHANGELOG.md` section. Archives must contain `agentskill`, `agsk`,
and `LICENSE`, and publication requires `SHA256SUMS` plus locked verification.

## Known Limits

Analyzer output is evidence, not a complete semantic understanding. Test
commands may be inferred from repository-level files, language classification
can be ambiguous for extensions such as `.m`, and fixture-heavy repositories
can dominate raw statistics. Inspect representative production files and CI
before asserting conventions. When evidence conflicts or is insufficient,
surface the uncertainty or ask a focused maintainer question.

## Provenance And Decisions

- Evidence Schema Version: `3`
- Repository Revision: `b9cee616e1bc27469459210e2305db82dd2b6ed9`
- Configuration: signature enabled by root `agentskill.toml`.
- Maintainer-Confirmed Decisions: None recorded.
- Unresolved Uncertainty: None recorded.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
