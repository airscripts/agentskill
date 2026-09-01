# Architecture

agentskill is a Rust evidence tool and an LLM skill. The Rust side discovers
facts that tools can verify; the skill turns those facts, source inspection, and
maintainer answers into useful repository guidance. The Rust CLI never writes
semantic Markdown. Generated documents are managed by the LLM skill;
maintainers place custom instructions in the root-level `## Free Region`,
which is preserved verbatim.

## System Boundary

The target repository is read, bounded, and never modified by an analyzer run.
The supported target-language matrix remains owned by the language registry.
The filesystem walker excludes dependency, cache, build, and generated trees,
sorts paths deterministically, classifies file roles, and caps file count and
text reads.

```text
agentskill-core/
  errors, language registry, filesystem snapshots, document parsing, output
agentskill-analyzers/
  scan, measure, config, git, graph, symbols, tests, normalized evidence
agentskill-generation/  (package: agentskill-validation)
  read-only AGENTS.md validation and drift checks
agentskill/
  Clap parsing, dispatch, JSON output, agentskill/agsk binaries
agentskill-skill/
  LLM workflow, output contract, references, and examples
```

Dependencies point inward: the CLI depends on analyzer and validation
libraries, analyzers depend on core, and the skill consumes the CLI contract.
The physical `agentskill-generation/` path is retained for source-layout
compatibility; its published crate name is `agentskill-validation`.

## Evidence Flow

```mermaid
flowchart LR
    repo[Target repository] --> walk[Bounded deterministic file walk]
    walk --> analyzers[Seven analyzers]
    analyzers --> raw[Stable analyzer JSON]
    walk --> facts[Normalized facts]
    raw --> facts
    facts --> skill[LLM skill + source inspection]
    skill --> docs[AGENTS.md and optional AGENTS.reference.md]
    docs --> checks[validate and drift]
```

The `analyze` command remains the broad raw analyzer contract. `evidence` is
the synthesis contract and includes:

```json
{
  "schema_version": 4,
  "agentskill_version": "2.0.0",
  "repository": {"root": "...", "revision": "...", "dirty": false},
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
  "budget": {"mode": "standard", "input_tokens": 8000, "output_tokens": 1000, "follow_up_rounds": 2},
  "scopes": [{
    "path": ".",
    "parent": ".",
    "status": "managed",
    "resolution": {
      "ancestors": [],
      "fallback": ".",
      "precedence": "nearest-scope-wins"
    }
  }],
  "scope_evidence": [{
    "path": ".",
    "parent": ".",
    "ancestors": [],
    "fallback": ".",
    "local_files": [],
    "inherited_files": [],
    "graph_files": [],
    "excluded_siblings": []
  }],
  "analyzers": {}
}
```

Every normalized fact has an identifier, scope, confidence, and provenance.
Facts should be compact and actionable: inventories, tools, commands, test
topology, language roles, and architectural boundaries. Raw analyzer output is
retained for deeper inspection, not copied into Markdown.

`scopes` is a deterministic manifest of existing and candidate guidance
boundaries. `scope_evidence` identifies local files, inherited support files,
graph-related files, and excluded siblings for each selected scope. Every scope
also exposes its ancestor chain, nearest managed fallback, and precedence.
Validation and drift report unsupported or low-confidence evidence, duplicate
inherited rules, and contradictory inherited rules; semantic rule authorship
remains with the LLM.

## Crate Responsibilities

### Core

`agentskill-core::fs` owns bounded repository walking and `RepoFile` metadata,
including source, test, example, documentation, generated, configuration, and
auxiliary roles. `language` is the source of truth for language and test-path
detection. `document` parses headings for validation; it is not a generation
engine. `output` owns stable JSON serialization and safe output paths.

### Analyzers

Each analyzer exposes a tolerant `run` function returning structured JSON or a
structured error. The aggregate runner dispatches the fixed registry in
parallel and inserts results in stable key order. The evidence adapter adds
normalized facts without changing the established analyzer keys.

| Analyzer | Primary signal |
| --- | --- |
| `scan` | tree, inventory, languages, read order |
| `measure` | indentation, line lengths, whitespace |
| `config` | formatters, linters, type checkers, project markers |
| `git` | history, branches, commit conventions |
| `graph` | internal imports, cycles, package boundaries |
| `symbols` | declarations and naming patterns |
| `tests` | frameworks, mappings, fixtures, commands |

### Validation

`agentskill-validation` does not infer or render repository rules. `validate`
checks that the LLM-authored documents exist, have unique headings, contain
valid local references and provenance, end cleanly, and keep the operational
file within its budget. It also reports unsupported facts, low-confidence facts,
and commands without repository support when those checks are possible from
static files. `drift` reruns evidence, captures the current version and
repository revision, and reports broken paths and stale versions in both
documents. A version mismatch means the guidance needs refreshing; a revision
mismatch is informational because repository commits do not necessarily change
guidance. Both operations are read-only. `drift` is advisory and returns
success when analysis completes,
even when findings exist.

The repository also provides reusable GitHub Actions in
`agentskill-actions/`. The `drift` Action installs a specified Agentskill
release, runs the advisory `drift` check, writes a job summary, and exposes the
JSON report path and stale status. The `validate` Action runs strict document
validation and exposes its JSON report path. Caller workflows under
`.github/workflows/` are responsible for triggers and artifact uploads. The
Actions install a pinned release by default and support source mode when a
repository workflow needs to test the checked-out CLI.

## LLM Document Contract

The skill owns `init`, `enrich`, `scope`, `update`, `audit`, and
`explain`.
`operational` and `reference` are depth views of one understanding, not two
independent generations.

The root `AGENTS.md` is normally 500–1,000 tokens and has a hard warning or
failure threshold at 1,500. It should contain mission/map, non-negotiables,
conceptual don'ts, quick-start commands, change routing, architecture,
implementation/testing rules, and a few high-value playbooks. Rules must be
source-backed or explicitly supplied by a maintainer. If a reference file
exists, the root links to `AGENTS.reference.md` and remains useful without it.

The reference document has no equivalent compact budget. It may contain deep
architecture, rationale, ownership, workflows, examples, history, uncertainty,
and evidence details. The skill should load it selectively; very large context
may be split into topic files while keeping the root index compact.

The Rust runtime exposes scope discovery, scoped evidence, validation, drift,
and fixed budget profiles, but never authors semantic Markdown. The LLM decides
whether to create or update each scope after comparing evidence with direct
source inspection and a semantic diff. Missing candidates are suggestions and
do not create files automatically.

## CLI Contracts

```text
agentskill <command> ...
agsk       <command> ...
```

Public commands are `analyze`, `evidence`, `scopes`, `scan`, `measure`, `config`,
`git`, `graph`, `symbols`, `tests`, `validate`, and `drift`. `evidence`,
`validate`, and `drift` accept repeatable `--scope PATH`; `evidence` accepts
`--budget compact|standard|deep`. JSON commands support `--pretty` and safe
relative `--out FILE`. Analyzer failures are JSON payloads
with a failed status; invalid arguments and unusable paths are process errors.

## Testing And Extension

Changes to analyzer keys, evidence fields, language detection, roles, command
flags, or validation semantics require contract tests and documentation.
Fixtures under `agentskill-skill/examples/` cover target languages, while
`agentskill-tests/fixtures/guidance/` covers document ownership,
signatures, provenance, and declined questions. Keep analyzer logic in
`agentskill-analyzers`, shared behavior in core, validation in its owning crate,
and the CLI entrypoint thin.

Adding a language updates the registry, representative fixture, applicable
analyzer tests, documentation, and release matrix coverage. Adding an analyzer
updates the registry, dispatch, evidence mapping when relevant, tests, and CLI
documentation.

## Release Architecture

Numeric `X.Y.Z` and `X.Y.Z-rc.N` tags drive verified GitHub Actions releases.
Archives contain both binaries and `LICENSE`; `SHA256SUMS` is required. `VERSION`
and the workspace version are `2.0.0`. Locked checks, archive
verification, and release-note extraction remain mandatory.
