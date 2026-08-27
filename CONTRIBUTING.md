# Contributing

## Development Setup

Install Rust through [rustup](https://rustup.rs/), then verify the workspace:

```bash
make verify
```

The minimum supported Rust version is 1.89. Keep `Cargo.lock` updated when
dependencies change.

## Architecture

- `agentskill-core` owns shared domain models, repository traversal, language
  detection, error payloads, and markdown document operations.
- `agentskill-analyzers` owns the seven analyzer families and their stable JSON
  output contracts.
- `agentskill-generation` owns deterministic markdown generation, references,
  profiles, layouts, interactive notes, and update merges.
- `agentskill` owns Clap parsing and the `agentskill`/`agsk` binaries only.

Keep implementation logic in the appropriate crate. Do not reintroduce runtime
Python or Python package-manager tooling. Python fixtures under
`agentskill-skill/examples/` are retained because Python is a supported target
language for analysis.

## Tests And Contracts

Add Rust unit tests beside the owning crate or integration tests under that
crate's `tests/` directory. Preserve command names, flags, output keys, error
payloads, generated section order, and update behavior unless a deliberate v2
contract change is documented.

## Documentation And Releases

Update `README.md`, `agentskill-skill/SKILL.md`, `agentskill-skill/SYSTEM.md`,
and `agentskill-docs/` when public CLI behavior changes. Add user-visible
changes to `CHANGELOG.md`. Stable
release tags must match `VERSION` and have a matching changelog heading;
`X.Y.Z-rc.N` tags publish prereleases automatically.

Use `make build`, `make coverage`, `make security`, or `make workflows` for
individual checks. Run `make fmt` to apply Rust formatting.
