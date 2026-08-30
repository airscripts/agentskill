# Agentskill Drift Action

This reusable GitHub Action installs a specified Agentskill release, verifies
its checksum, and runs the advisory `drift` check against the checked-out
repository.

```yaml
name: Agentskill

on:
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7

      - id: agentskill
        uses: airscripts/agentskill/agentskill-actions/drift@<commit-sha>
        with:
          version: 2.0.0

      - uses: actions/upload-artifact@v7
        if: always()
        with:
          name: agentskill-drift-report
          path: ${{ steps.agentskill.outputs.report-path }}
```

The optional `version` input should point to a published Agentskill release
unless `source: "true"` is used. The optional `signature` input accepts `auto`,
`on`, or `off`. Findings appear in the job summary and JSON output. The summary
explains that the check is advisory and points to the full report. Source mode
expects an `agentskill` binary to already be available on `PATH`.
