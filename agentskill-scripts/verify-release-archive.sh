#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <archive> <target>" >&2
    exit 2
fi

archive="$1"
target="$2"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

case "$archive" in
    *.tar.gz) tar -xzf "$archive" -C "$workdir" ;;
    *.zip) unzip -q "$archive" -d "$workdir" ;;
    *) echo "unsupported archive format: $archive" >&2; exit 1 ;;
esac

if [[ "$target" == *windows* ]]; then
    binaries=(agentskill.exe agsk.exe)
else
    binaries=(agentskill agsk)
fi

for required in "${binaries[@]}" LICENSE; do
    if ! find "$workdir" -type f -name "$required" | grep -q .; then
        echo "archive $archive is missing $required" >&2
        exit 1
    fi
done

