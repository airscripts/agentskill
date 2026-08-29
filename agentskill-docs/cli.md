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
trailing newlines, and the operational token budget. `drift` records the current
repository revision and reports broken paths referenced by the documents. Both
commands are read-only and return a non-zero status when their check fails.

## Exit Status And Output

Successful commands write JSON and exit `0`. Analyzer failures are represented
as JSON error payloads and exit `1`. Invalid paths and output failures are
reported on stderr and exit `1`. `--out` accepts a safe relative output path;
without it, JSON is written to stdout.

Use `agentskill --help` and `agentskill <command> --help` for the exact syntax.
