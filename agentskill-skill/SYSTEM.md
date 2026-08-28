# SYSTEM.md — Agentskill Generation Contract

This document is the behavioral source of truth for generated `AGENTS.md`
files. Generated guidance must be grounded in repository evidence and must
allow an agent to produce code consistent with the analyzed repository.

## Required Sections

Generate these sections in this order when evidence is available:

1. Overview
2. Repository Structure
3. Service Map
4. Cross-Service Boundaries
5. Commands and Workflows
6. Code Formatting
7. Naming Conventions
8. Type Annotations
9. Imports
10. Error Handling
11. Comments and Docstrings
12. Testing
13. Git
14. Dependencies and Tooling
15. Red Lines

Omit sections that have no applicable evidence. Scope language-specific rules
under the relevant language or service and never apply one ecosystem's rules to
another. Preserve concrete examples when they are necessary to make a rule
operational.

## Generation Modes

The Rust CLI provides deterministic `generate` and `update` workflows. This
packaged skill provides AI-assisted synthesis: the model reads analyzer JSON,
source files, configuration, tests, and this specification before writing the
final document. The model must not delegate AI-authored output to the static
generator.

`generate` creates a fresh document. `update` parses the existing document,
regenerates selected sections, preserves untouched custom content, and supports
`--force` for a clean rebuild. Profiles are `concise` and `comprehensive`;
layouts are `single`, `split`, and `multifile` for generation, while update is
single-file only.

## Quality Requirements

- Never invent commands, tools, file paths, or conventions.
- Prefer source-backed rules over analyzer statistics.
- Surface unresolved ambiguity or preserve existing manual text.
- Keep markdown headings, links, code fences, and trailing newline behavior
  valid and deterministic.
- Update this specification whenever generation semantics change.
