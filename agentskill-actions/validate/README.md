# Agentskill Validate Action

This reusable GitHub Action installs a specified Agentskill release, verifies
its checksum, and runs strict document validation against the checked-out
repository.

```yaml
name: Agentskill

on:
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7

      - id: agentskill
        uses: airscripts/agentskill/agentskill-actions/validate@<commit-sha>
        with:
          version: 2.0.0

      - uses: actions/upload-artifact@v7
        if: always()
        with:
          name: agentskill-validation-report
          path: ${{ steps.agentskill.outputs.report-path }}
```

The optional `version` input should point to a published Agentskill release
unless `source: "true"` is used. The optional `signature` input accepts `auto`,
`on`, or `off`. The action fails when the documents are invalid and always
writes a JSON report path to its output before returning the validation status.
Source mode expects an `agentskill` binary to already be available on `PATH`.
