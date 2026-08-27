# Agentskill Release Scripts

These scripts are used by the reusable GitHub Actions release workflows.

- `release-notes.sh` validates numeric release tags and extracts final notes
  from `CHANGELOG.md`.
- `verify-release-archive.sh` checks that every archive contains both CLI
  binaries and the MIT license.
- `pre-commit.sh` runs filtered Rust checks for changed crates and uses a
  disposable Rust container when the required local toolchain is unavailable.

## Local Validation

Install `actionlint` and `shellcheck`, then run:

```bash
actionlint -color
shellcheck --shell=bash agentskill-scripts/*.sh
```
