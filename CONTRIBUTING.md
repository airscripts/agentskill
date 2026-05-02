# Contributing

When contributing to this repository, please first discuss the change you want
to make through an issue, discussion, or pull request draft where appropriate.

Before contributing, read and follow:

- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [README.md](./README.md)
- [AGENTS.md](./AGENTS.md)

## What To Contribute

Useful contribution areas include:

- Analyzer accuracy improvements.
- Richer static `AGENTS.md` generation.
- Language support expansion.
- CLI and output contract improvements.
- Tests, fixtures, and regression coverage.
- Documentation and skill workflow clarity.

## Development Workflow

Set up the local environment:

```bash
python -m pip install -e '.[dev]'
pre-commit install
```

Run the standard checks before opening a pull request:

```bash
ruff format .
ruff check .
mypy
pytest
```

## Pull Requests

- Keep changes focused and reviewable.
- Add or update tests when behavior changes.
- Update docs when the CLI, skill workflow, or generated output semantics change.
- Preserve the packaged/runtime split described in `README.md` and `AGENTS.md`.
- Prefer deterministic behavior and contract-stable output when changing generation code.

## Issues

Use the repository issue templates for:

- Bug reports.
- Feature requests.
- Documentation gaps.

Include reproduction steps, expected behavior, and actual behavior whenever
possible.
