#!/usr/bin/env bash

set -euo pipefail

if [[ -z "${AGENTSKILL_VERSION:-}" ]]; then
  echo "Agentskill version is required when source mode is disabled." >&2
  exit 1
fi

case "$RUNNER_OS" in
  Linux)
    case "$RUNNER_ARCH" in
      X64) target="x86_64-unknown-linux-gnu" ;;
      ARM64) target="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported Linux runner architecture: $RUNNER_ARCH" >&2; exit 1 ;;
    esac
    ;;
  macOS)
    case "$RUNNER_ARCH" in
      X64) target="x86_64-apple-darwin" ;;
      ARM64) target="aarch64-apple-darwin" ;;
      *) echo "Unsupported macOS runner architecture: $RUNNER_ARCH" >&2; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported Agentskill runner operating system: $RUNNER_OS" >&2; exit 1 ;;
esac

release_version="${AGENTSKILL_VERSION%-rc.*}"
package="agentskill-${release_version}-${target}"
archive_path="${RUNNER_TEMP}/${package}.tar.gz"
checksums_path="${RUNNER_TEMP}/agentskill/SHA256SUMS"
install_dir="${RUNNER_TEMP}/agentskill/bin"
mkdir -p "$install_dir"

curl --fail --silent --show-error --location \
  "https://github.com/airscripts/agentskill/releases/download/${AGENTSKILL_VERSION}/${package}.tar.gz" \
  --output "$archive_path"
curl --fail --silent --show-error --location \
  "https://github.com/airscripts/agentskill/releases/download/${AGENTSKILL_VERSION}/SHA256SUMS" \
  --output "$checksums_path"

expected_checksum="$(awk -v package="$package" '$2 == package {print $1}' "$checksums_path")"
test -n "$expected_checksum"
printf '%s  %s\n' "$expected_checksum" "$archive_path" | sha256sum --check --status -

tar -xzf "$archive_path" -C "$RUNNER_TEMP"
install -m 0755 "${RUNNER_TEMP}/${package}/agentskill" "$install_dir/agentskill"
echo "$install_dir" >> "$GITHUB_PATH"
