# AGENTS.md -- Coding Style

## Overview

Pragmatic tool-builder. Self-documenting code over verbose comments, descriptive function names over terse ones, and fail-fast over defensive error handling. Conventional commit discipline with lean, action-oriented messages. Prefers clarity over cleverness.

---

## Cross-Language Patterns

### Naming
- **Variables:** Mixed case -- `camelCase` preferred, `snake_case` acceptable
- **Types:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Functions:** Highly descriptive (20+ chars avg), `snake_case`

### Comments
- **Density:** Minimal (~5% of code lines)
- **Style:** `//` for inline, `///` for doc comments, `"""` for Python docstrings
- **Philosophy:** Code explains itself; comments reserved for "why"

### Error Handling
- CLI-style: `unwrap` acceptable for user-facing tools
- Library code: propagate with `?`, never `unwrap`

---

## Rust

### Naming
- Variables: Mixed case observed
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE` (115) over `PascalCase` (8)
- Functions: Highly descriptive -- 22 char avg, `snake_case`

### Error Handling
- 254 `unwrap()` calls alongside 232 `?` propagations
- No `expect()` usage
- Zero `panic!`

### Comments
- Doc comments: `///` for public API, `//!` for module-level
- Normal comments: `//` for inline

### Imports
- Prefer `use` with granular paths
- External crates tracked via `Cargo.toml`

### Tooling
- Cargo workspace
- GitHub Actions CI

---

## Python

### Naming
- Variables: `snake_case`
- Functions: `snake_case`
- Classes: `PascalCase`

### Error Handling
- Prefer `try/except` for expected failures
- Use `raise` for explicit errors
- `with` statements for resource management

### Comments
- Docstrings: `"""` for modules and public functions
- Normal comments: `#` for inline

---

## Go

### Naming
- Variables: `camelCase` for unexported, `PascalCase` for exported
- Functions: `PascalCase` for exported, `camelCase` for unexported

### Comments
- `//` for both inline and doc comments

---

## Git

### Commits
- **Format:** Conventional commits
- **Prefixes:** `feat`, `fix`, `docs`, `refactor`, `chore`
- **Length:** Descriptive -- 41 chars avg
- **Style:** Lowercase, imperative mood

### Branches
- Sparse branching -- trunk-based workflow

### Signing
- GPG signing: detect from git config
- Signoff: detect from git config

### Remotes
- Primary: GitHub
- Remote count varies by project

---

## Tooling

- GitHub Actions CI
- No explicit formatter configs detected
- Lockfiles: detected per language (Cargo.lock, package-lock.json, etc.)
- Editor config: detected if `.editorconfig` present

---

## Red Lines

- Never `panic!` -- controlled failure only
- No verbose comments -- self-document or omit
- Fail fast over defensive in CLI context
- Descriptive function names -- clarity over brevity
- Never mix naming conventions within a category

---

**Source:** Sample repository (68 commits, 40 Rust files)

**Confidence:** High on naming/git patterns; Medium on branch conventions (limited sample)