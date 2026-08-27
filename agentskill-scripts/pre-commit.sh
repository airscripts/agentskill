#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

required_rust="${AGENTSKILL_RUST_VERSION:-1.89}"
rust_image="${AGENTSKILL_RUST_IMAGE:-rust:${required_rust}-bookworm}"
in_container="${AGENTSKILL_PRECOMMIT_CONTAINER:-0}"
files=("$@")

if [[ ! "$required_rust" =~ ^[0-9]+\.[0-9]+$ ]]; then
    echo "AGENTSKILL_RUST_VERSION must use major.minor format: $required_rust" >&2
    exit 2
fi

if [ "${#files[@]}" -eq 0 ]; then
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git diff --cached --name-only -z --diff-filter=ACMR)
fi

packages=""
all_packages=0

add_package() {
    case " $packages " in
        *" $1 "*) ;;
        *) packages="$packages $1" ;;
    esac
}

for file in "${files[@]}"; do
    case "$file" in
        Cargo.toml|Cargo.lock)
            all_packages=1
            ;;
        agentskill/Cargo.toml|agentskill/src/*|agentskill/tests/*)
            add_package agentskill
            ;;
        agentskill-core/Cargo.toml|agentskill-core/src/*|agentskill-core/tests/*)
            add_package agentskill-core
            ;;
        agentskill-analyzers/Cargo.toml|agentskill-analyzers/src/*|agentskill-analyzers/tests/*)
            add_package agentskill-analyzers
            ;;
        agentskill-generation/Cargo.toml|agentskill-generation/src/*|agentskill-generation/tests/*)
            add_package agentskill-generation
            ;;
    esac
done

if [ "$all_packages" -eq 1 ]; then
    packages=" agentskill agentskill-core agentskill-analyzers agentskill-generation"
fi

if [ -z "$packages" ]; then
    exit 0
fi

rust_is_compatible() {
    command -v rustc >/dev/null 2>&1 || return 1

    local installed major minor required_major required_minor
    installed="$(rustc --version | awk '{print $2}')"
    required_major="${required_rust%%.*}"
    required_minor="${required_rust#*.}"

    if [[ ! "$installed" =~ ^([0-9]+)\.([0-9]+) ]]; then
        return 1
    fi

    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    ((major > required_major || (major == required_major && minor >= required_minor)))
}

if ! rust_is_compatible; then
    if [ "$in_container" = "1" ]; then
        echo "Rust $required_rust or newer is required" >&2
        exit 1
    fi

    if ! command -v docker >/dev/null 2>&1; then
        echo "Rust $required_rust or newer is required, and Docker is unavailable" >&2
        exit 1
    fi

    exec docker run --rm \
        --env AGENTSKILL_PRECOMMIT_CONTAINER=1 \
        --env AGENTSKILL_RUST_VERSION="$required_rust" \
        --user "$(id -u):$(id -g)" \
        --volume "$root:/workspace" \
        --workdir /workspace \
        "$rust_image" \
        bash -c 'rustup component add rustfmt clippy && exec bash agentskill-scripts/pre-commit.sh "$@"' \
        -- "${files[@]}"
fi

for package in $packages; do
    cargo fmt --package "$package" -- --check
done

for package in $packages; do
    cargo clippy --package "$package" --all-targets --locked -- -D warnings
done

for package in $packages; do
    cargo test --package "$package" --locked
done
