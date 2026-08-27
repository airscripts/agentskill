# CLI Reference

Both `agentskill` and `agsk` expose the same command surface:

```text
analyze <repo>...
scan <repo>
measure <repo>
config <repo>
git <repo>
graph <repo>
symbols <repo>
tests <repo>
generate <repo>
update <repo>
```

Analyzer commands accept `--lang` where applicable, `--pretty`, and `--out
FILE`. `analyze` accepts multiple repositories and repeatable `--reference`
flags. `generate` accepts `--reference`, `--interactive`, `--profile`, and
`--layout`. `update` accepts `--section`, `--exclude-section`, `--force`, and
`--profile`; only the `single` layout is supported for updates.

Use `agentskill --help` and `agentskill <command> --help` for the exact syntax.
