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

---

**Source:** my-cli-tool
**Files analyzed:** 18
**Confidence:** High