#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH="" cd "$SCRIPT_DIR/../.." && pwd)"
VALIDATOR="$REPO_ROOT/scripts/validate-tech-stack.sh"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/technology-harness-test.XXXXXX")"
trap 'rm -rf "$FIXTURE"' EXIT

git -C "$FIXTURE" init -q
mkdir -p "$FIXTURE/.specify/memory" "$FIXTURE/specs/001-sample"
printf '%s\n' '# Constitution' 'No technology defaults.' \
    > "$FIXTURE/.specify/memory/constitution.md"
printf '%s\n' '# Specification' \
    'Implement a Rust service using Tokio and PostgreSQL.' \
    > "$FIXTURE/specs/001-sample/spec.md"
printf '%s\n' '# Architecture' 'One bounded sample capability.' \
    > "$FIXTURE/specs/001-sample/architecture.md"

expect_failure() {
    local expected="$1"
    shift
    local output

    if output="$("$@" 2>&1)"; then
        echo "Expected command to fail: $*" >&2
        exit 1
    fi

    if [[ "$output" != *"$expected"* ]]; then
        echo "Expected failure output to contain: $expected" >&2
        echo "$output" >&2
        exit 1
    fi
}

expect_failure "TECHNOLOGY CLARIFICATION REQUIRED" \
    "$VALIDATOR" --feature-dir "$FIXTURE/specs/001-sample" --phase plan

printf '%s\n' \
    'language: Rust' \
    'runtime: Tokio' \
    'testing: cargo test' \
    > "$FIXTURE/specs/001-sample/tech-stack.yaml"

expect_failure "Undeclared technology detected:" \
    "$VALIDATOR" --feature-dir "$FIXTURE/specs/001-sample" --phase tasks

printf '%s\n' \
    'language: Rust' \
    'runtime: Tokio' \
    'testing: cargo test' \
    'databases:' \
    '  - PostgreSQL' \
    > "$FIXTURE/specs/001-sample/tech-stack.yaml"

output="$(
    "$VALIDATOR" \
        --feature-dir "$FIXTURE/specs/001-sample" \
        --phase implement
)"

if [[ "$output" != *"Technology declaration validation: PASS"* ]]; then
    echo "Expected successful validation" >&2
    echo "$output" >&2
    exit 1
fi

mkdir -p "$FIXTURE/specs/000-parent" "$FIXTURE/specs/002-child"
printf '%s\n' '# Capability' 'A parent capability.' \
    > "$FIXTURE/specs/000-parent/spec.md"
printf '%s\n' '# Architecture' 'Events use Kafka.' \
    > "$FIXTURE/specs/000-parent/architecture.md"
printf '%s\n' '# Index' '- **Specification ID**: 002-child' \
    > "$FIXTURE/specs/000-parent/feature-index.md"
printf '%s\n' '# Child Specification' 'A bounded child.' \
    > "$FIXTURE/specs/002-child/spec.md"
printf '%s\n' \
    'language: Rust' \
    'runtime: Tokio' \
    'testing: cargo test' \
    > "$FIXTURE/specs/002-child/tech-stack.yaml"

expect_failure "Kafka" \
    "$VALIDATOR" --feature-dir "$FIXTURE/specs/002-child" --phase plan

printf '%s\n' \
    'language: Rust' \
    'runtime: Tokio' \
    'testing: cargo test' \
    'transports: [Kafka]' \
    > "$FIXTURE/specs/002-child/tech-stack.yaml"

"$VALIDATOR" \
    --feature-dir "$FIXTURE/specs/002-child" \
    --phase plan \
    >/dev/null

echo "Technology declaration harness tests: PASS"
