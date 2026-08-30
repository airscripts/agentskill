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
  detection, file roles, error payloads, and document parsing.
- `agentskill-analyzers` owns the seven analyzer families and their stable JSON
  output contracts.
- `agentskill-validation` owns read-only AGENTS.md validation and evidence/drift
  checks (its source directory remains `agentskill-generation/`).
- `agentskill` owns Clap parsing and the `agentskill`/`agsk` binaries only.

Keep implementation logic in the appropriate crate. Do not reintroduce runtime
Python or Python package-manager tooling. Python fixtures under
`agentskill-skill/examples/` are retained because Python is a supported target
language for analysis.

## Repository Scopes

In this repository, a commit scope identifies an owning submodule. Use the
submodule name without the common `agentskill-` prefix: `core`, `analyzers`,
`generation`, `cli`, `skill`, `docs`, `actions`, `tests`, `scripts`, `assets`,
or `specs`. Route implementation, tests, and documentation changes to the
submodule that owns the behavior. Omit the scope for root-level or
cross-submodule changes.

Guidance is a product area, not a commit scope. Use “guidance” to describe the
managed `AGENTS.md` feature, but use the owning submodule name in commit
subjects.

Documentation-only changes use the `docs:` conventional commit without a
scope, including changes under `agentskill-docs/`. For example, use
`docs: document guidance actions`, not `docs(docs): document guidance actions`.

## Tests And Contracts

Add Rust unit tests beside the owning crate or integration tests under that
crate's `tests/` directory. Preserve command names, flags, output keys, error
payloads, evidence provenance, and validation behavior unless a deliberate v2
contract change is documented.

## Documentation And Releases

Update `README.md`, `agentskill-skill/SKILL.md`, `agentskill-skill/SYSTEM.md`,
and `agentskill-docs/` when public CLI behavior changes. Add user-visible
changes to `CHANGELOG.md`. Stable
release tags must match `VERSION` and have a matching changelog heading;
`X.Y.Z-rc.N` tags publish prereleases automatically.

Use `make build`, `make coverage`, `make security`, or `make workflows` for
individual checks. Run `make fmt` to apply Rust formatting.
