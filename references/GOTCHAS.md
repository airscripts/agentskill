# GOTCHAS.md -- Errors to Avoid in Generation

## Extraction Errors

| Error | Problem | Fix |
|-------|---------|-----|
| Keyword pollution | `self`, `if`, `for` counted as vars | Filter keywords in PYTHON_VAR_KEYWORDS |
| Single-word ambiguity | `foo` matches camelCase and snake_case | Require case transitions |
| Generated file skew | lockfiles have 0 comments | Exclude vendored/generated via SKIP_DIRS |
| Test file bias | tests use `unwrap` heavily | Sample src/ and tests/ separately |
| Git squash gaps | lost commit granularity | Note when history is flattened |
| Naming dominance | Report counts per case style, not just "mixed" | RustAnalyzer/PythonAnalyzer track dominant_case |
| Import misclassification | stdlib modules counted as third-party | PythonAnalyzer checks known stdlib names |
| Branch inflation | remote branches double-counted | extract_branch_prefixes strips `remotes/origin/` |

## Synthesis Errors

| Error | Problem | Fix |
|-------|---------|-----|
| Overfitting | Pattern only in one repo | Flag "project-specific" |
| Default assumption | Claiming rustfmt use when it's cargo default | Only report explicit custom configs |
| Convention vs preference | `feat:` prefix might be team, not personal | Check solo projects |
| Context confusion | High unwrap in CLI != library tolerance | Contextualize by crate type |
| Universal noise | "Uses snake_case in Rust" -- obvious | Only document deviations from language defaults |
| Missing metadata | No project name or license detected | Check README.md and LICENSE files |
| Stale remotes | GitHub remote doesn't mean CI exists | Check `.github/workflows/` explicitly |

## Confidence Annotations

- **High** -- Consistent across repos, many files
- **Medium** -- Pattern with exceptions, or limited sample
- **Low** -- Few instances, flag as tentative

## Red Flags

- Generic advice applicable to anyone
- Contradictory patterns unexplained
- No src/test distinction
- Unusual claims without confidence annotations
- Reporting language defaults as personal style (e.g., "uses snake_case in Rust")