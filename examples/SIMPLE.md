# AGENTS.md — Coding Style

## Red Lines

- Never swallow errors silently — always propagate or log
- No `TODO` without a ticket reference
- No mutable global state

## Overview

Python CLI project. Pragmatic, fail-fast. Descriptive names, type hints on public APIs, minimal comments.

## Naming

- **vars:** snake_case
- **functions:** snake_case
- **types:** PascalCase
- **consts:** SCREAMING_SNAKE_CASE

## Error Handling

- `raise` with context — never bare exceptions
- Fail fast in CLI entry points

## Comments

- **Density:** Low (~3%)
- **Style:** `"""` for modules and public functions only
- Self-documenting code preferred over inline comments

## Imports

- **Order:** stdlib → third-party → local
- Use `isort` profile

## Repository Structure

```
my-cli-tool/
├── src/
│   ├── __init__.py
│   ├── cli.py
│   ├── commands/
│   │   ├── __init__.py
│   │   └── run.py
│   └── utils.py
├── tests/
│   ├── conftest.py
│   └── test_cli.py
├── pyproject.toml
└── requirements.txt
```

## Commands and Workflows

### Install
```bash
pip install -e ".[dev]"
```

### Test
```bash
pytest
```

### Lint
```bash
ruff check .
```

### Format
```bash
ruff format .
```

## Git

- **Commits:** conventional (`feat`, `fix`, `docs`, `chore`), lowercase, imperative
- **Branches:** `main` + `fix/` and `feat/` prefixes

## Tooling

- ruff (lint + format)
- pytest
- GitHub Actions CI

## Dependencies

- **Manager:** pip / uv
- **Pin style:** locked (requirements.txt)

## Code Examples

Actual patterns from the codebase:

### Example 1
```python
# From cli.py
@click.command()
def main(config_path: str) -> None:
    cfg = Config.load(config_path)
    result = run_pipeline(cfg)
    if result.errors:
        raise SystemExit(1)
```

### Example 2
```python
# From utils.py
def parse_input(raw: str) -> dict[str, Any]:
    """Parse raw input into structured data."""
    try:
        return json.loads(raw)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid input: {e}") from e
```

---

**Source:** my-cli-tool
**Files analyzed:** 18
**Confidence:** High