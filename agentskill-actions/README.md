# Agentskill Actions

This directory contains reusable GitHub Actions for Agentskill.

## Drift

Use [`drift/`](drift/) to run an advisory drift check. The action reports
findings without failing the workflow.

## Validate

Use [`validate/`](validate/) to run the strict document validation check. The
action fails when the repository documents are invalid.

Both actions require a checked-out repository. Release mode requires a pinned
Agentskill release tag; the repository's own workflow uses source mode to test
the checked-out CLI without depending on a published release.
