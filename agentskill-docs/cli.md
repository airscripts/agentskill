# CLI Reference

Both `agentskill` and `agsk` expose the same read-only command surface:

```text
analyze <repo>...
evidence <repo>
scopes <repo>
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

Analyzer commands accept `--pretty` and `--out FILE`; language-aware commands
also accept `--lang`. `analyze` accepts multiple repositories. `evidence`,
`validate`, and `drift` accept repeatable `--scope PATH`. The CLI never authors
or updates semantic Markdown.

## Evidence Workflow

`evidence` is the primary input for the LLM skill:

```bash
agentskill evidence <repo> --pretty
agentskill evidence <repo> --scope packages/api --budget compact --pretty
```

It returns schema version 4 with the Agentskill version, repository revision
metadata, normalized facts, confidence, evidence paths, scope metadata, local
and inherited file sets, and analyzer output. Facts are intentionally compact
and suitable for synthesis; source files and maintainer decisions still need
to be inspected by the skill.

`scopes` discovers existing nested `AGENTS.md` files and high-confidence
project boundaries without writing. Missing candidates are informational; an
LLM workflow creates a scoped document only after explicit selection. Each
entry also reports its ancestor chain, nearest managed fallback, and
`nearest-scope-wins` precedence.

Budget profiles are deterministic model-facing limits:

| Mode | Input context | Output | Follow-up rounds |
| --- | ---: | ---: | ---: |
| `compact` | 4,000 | 512 | 1 |
| `standard` | 8,000 | 1,000 | 2 |
| `deep` | 16,000 | 2,000 | 4 |

Compact evidence keeps high-confidence operational facts and trims low-priority
raw analyzer detail first. Standard is the default; Deep permits broader
reference loading. These modes do not detect hardware or guarantee throughput.

## Document Checks

After the LLM writes `AGENTS.md` and, optionally, `AGENTS.reference.md`, run:

```bash
agentskill validate <repo> --pretty
agentskill drift <repo> --pretty
agentskill validate <repo> --scope packages/api --signature auto
agentskill drift <repo> --scope packages/api --signature auto
```

`validate` checks document presence, duplicate headings, local references,
trailing newlines, provenance, supported commands, and the operational token
budget. `validate` and `drift` include all existing managed scopes by default;
`--scope` limits checks to the selected scope. Missing candidates are
informational and do not fail validation. `drift` records the current
Agentskill version and repository revision
and reports broken paths, stale versions, changed revisions, unsupported facts,
low-confidence facts, commands without repository support, duplicated inherited
rules, and conflicting inherited rules. Version and conflict findings make the
report stale; revision and duplicate-rule findings are informational. Both
commands are read-only. `validate` returns a non-zero status for invalid documents; `drift`
is advisory and returns zero when analysis completes, even when it reports
findings. Both commands accept
`--signature auto|on|off`.

Signatures are enabled by default. Set `signature = false` in the root
`agentskill.toml` to disable them for the repository, or pass `--signature on`
or `--signature off` for one check without changing the file. Commands are
checked only when their repository support can be confirmed from static files;
the checks never run commands.

Repositories can run document checks in CI with the reusable GitHub Actions in
`agentskill-actions/`:

```yaml
- uses: airscripts/agentskill/agentskill-actions/drift@<commit-sha>
  with:
    version: 2.1.0
```

Use `agentskill-actions/validate` for strict validation. Release mode requires a
checked-out repository and a pinned release; source mode uses the CLI built from
the checked-out source. The caller can upload the JSON report using the Action's
`report-path` output; drift remains advisory, while validate fails when the
documents are invalid.

```yaml
- uses: airscripts/agentskill/agentskill-actions/validate@<commit-sha>
  with:
    version: 2.1.0
```

Repository workflows can set `source: "true"` after building the checked-out
CLI. This skips release installation and tests the current source.

## Exit Status And Output

Successful commands write JSON and exit `0`. Analyzer failures are represented
as JSON error payloads and exit `1`. Invalid paths and output failures are
reported on stderr and exit `1`. `--out` accepts a safe relative output path;
without it, JSON is written to stdout.

Use `agentskill --help` and `agentskill <command> --help` for the exact syntax.
