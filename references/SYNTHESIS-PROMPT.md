# Synthesis Prompt

Synthesize AGENTS.md from extraction report. Multi-language.

## Input

```json
{
  "repos": [...],
  "analyses": [{
    "path": "/abs/path",
    "languages": {
      "rust": {
        "naming": {"vars": {"dominant_case": "snake_case", "counts": {...}}, ...},
        "error_handling": {"unwrap": 5, "?": 10, ...},
        "comments": {"doc_comments": 3, "normal_comments": 8, "density": 0.12},
        "spacing": {"avg_blank_lines": 1.3},
        "imports": {"std": 5, "crate": 2, "external": 1},
        "file_count": 10
      },
      "python": { ... }
    },
    "git": {
      "commits": {"count": 50, "avg_length": 42, "common_prefixes": {"feat": 20, "fix": 15}},
      "branches": {"count": 5, "common_prefixes": {"feature": 3}}
    },
    "tooling": {"cargo": true, "GitHub Actions CI": true},
    "metadata": {"project_name": "...", "has_license": true, "license_type": "MIT"}
  }]
}
```

## Output Structure

```markdown
# AGENTS.md -- Coding Style

## Overview
One-paragraph philosophy.

## Cross-Language Patterns
Patterns holding across langs (if any).

## [Language]
### Naming
dominant case style per category, with counts

### Error Handling
language-specific patterns with occurrence counts

### Comments
density, doc style, philosophy

### Spacing
blank line habits

## Git
- Commits: format, prefixes, length
- Branches: naming patterns
- Signing: GPG, signoff
- Remotes: GitHub, GitLab

## Tooling
Detected configs and lockfiles.

## Red Lines
Explicit avoidances derived from patterns.

---
**Source:** repo names + file stats
**Confidence:** High/Medium/Low per claim
```

## Rules

1. **Multi-language.** Document every language found.
2. **Cross-language top.** Shared patterns once at top.
3. **Data only.** Omit if ambiguous; annotate confidence.
4. **Flag project-specific.** Single-repo patterns labeled.
5. **No universal noise.** Skip obvious (e.g., snake_case in Rust).
6. **Contextualize.** unwrap in CLI != library.
7. **Read GOTCHAS.md first.**
8. **Include metadata.** Project name, license type, remote info when available.
9. **Dominant case.** Report the dominant naming style with counts, not just "mixed".