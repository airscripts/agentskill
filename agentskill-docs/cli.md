# CLI Reference

Both `agentskill` and `agsk` expose the same read-only command surface:

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

Analyzer commands accept `--pretty` and `--out FILE`; language-aware commands
also accept `--lang`. `analyze` accepts multiple repositories. The CLI never
authors or updates semantic Markdown.

## Evidence Workflow

`evidence` is the primary input for the LLM skill:

```bash
agentskill evidence <repo> --pretty
```

It returns a versioned bundle containing repository revision metadata,
normalized facts, confidence, evidence paths, and the complete analyzer output.
Facts are intentionally compact and suitable for synthesis; source files and
maintainer decisions still need to be inspected by the skill.

## Document Checks

After the LLM writes `AGENTS.md` and, optionally, `AGENTS.reference.md`, run:

```bash
agentskill validate <repo> --pretty
agentskill drift <repo> --pretty
```

`validate` checks document presence, duplicate headings, local references,
trailing newlines, provenance, supported commands, and the operational token
budget. `drift` records the current repository revision and reports broken
paths, stale provenance, unsupported facts, low-confidence facts, and commands
without repository support. Both commands are read-only. `validate` returns a
non-zero status for invalid documents; `drift` is advisory and returns zero when
analysis completes, even when it reports findings. Both commands accept
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

Use `agentskill-actions/validate` for strict validation. Both Actions require a
checked-out repository and a pinned release. The caller can upload the JSON
report using the Action's `report-path` output; drift remains advisory, while
validate fails when the documents are invalid.

```yaml
- uses: airscripts/agentskill/agentskill-actions/validate@<commit-sha>
  with:
    version: 2.1.0
```

## Exit Status And Output

Successful commands write JSON and exit `0`. Analyzer failures are represented
as JSON error payloads and exit `1`. Invalid paths and output failures are
reported on stderr and exit `1`. `--out` accepts a safe relative output path;
without it, JSON is written to stdout.

Use `agentskill --help` and `agentskill <command> --help` for the exact syntax.
