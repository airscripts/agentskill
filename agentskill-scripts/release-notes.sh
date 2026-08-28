#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <tag> <output-file>" >&2
    exit 2
fi

tag="$1"
output_file="$2"

if [[ ! "$tag" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
    echo "release tag must use X.Y.Z or X.Y.Z-rc.N format: $tag" >&2
    exit 1
fi

version="${tag%-rc.*}"
declared_version="$(tr -d '[:space:]' < VERSION)"

if [ "$version" != "$declared_version" ]; then
    echo "release tag $tag does not match VERSION $declared_version" >&2
    exit 1
fi

if [[ "$tag" == *-rc.* ]]; then
    cat > "$output_file" <<EOF
Release candidate for \`$version\`.

This prerelease is intended to validate the Rust CLI and platform archives.
See the final \`$version\` release for changelog notes.
EOF
    exit 0
fi

awk -v version="$version" -v tag="$tag" '
    $0 ~ "^## \\[" version "\\]" {
        found = 1
        first = 1
        date = $0
        sub("^## \\[" version "\\] - ", "", date)
        print "## agentskill@" tag " | " date
        print ""
        next
    }
    found && /^## \[/ { exit }
    found {
        if (first && $0 == "") { first = 0; next }
        first = 0
        print
    }
    END { if (!found) exit 1 }
' CHANGELOG.md > "$output_file"

if [ ! -s "$output_file" ]; then
    echo "CHANGELOG.md section for $version is empty or missing" >&2
    exit 1
fi
