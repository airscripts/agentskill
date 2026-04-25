# AGENTS.md Output Template

## Structure

1. **Overview** -- 1-2 sentence philosophy distilled from patterns
2. **Cross-Language Patterns** -- naming, comments, error handling, spacing shared across languages
3. **Per-Language Sections** -- one per detected language:
   - Naming (dominant case style per category, counts)
   - Error Handling (language-specific patterns with counts)
   - Comments (density, doc vs normal, style)
   - Spacing (average blank lines between blocks)
   - Imports (stdlib/crate/external ratios)
4. **Git** -- commit prefixes, average length, branch naming, signing, remotes
5. **Tooling** -- detected configs, lockfiles, CI, test frameworks
6. **Red Lines** -- explicit avoidances derived from pattern analysis
7. **Footer** -- source repos, file count, confidence annotations

## Section Order

Overview -> Cross-Language -> Languages (alphabetical) -> Git -> Tooling -> Red Lines -> Footer

## Synthesis Modes

- **Default** (`agentskill repo/`) -- generates full AGENTS.md
- **JSON** (`--json`) -- raw analysis data for custom processing
- **Skip sections** (`--skip-git`, `--skip-tooling`) -- omit sections

## Tone

Direct, imperative. Match commit message tone. No universal noise (skip "uses snake_case in Rust").