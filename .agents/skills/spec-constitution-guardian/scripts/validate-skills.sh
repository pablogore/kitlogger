#!/bin/sh
set -eu

root=${1:-.agents/skills}
failed=0

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  failed=1
}

for skill in "$root"/*/SKILL.md; do
  [ -f "$skill" ] || continue

  first=$(sed -n '1p' "$skill")
  [ "$first" = "---" ] || fail "$skill: missing YAML frontmatter"

  grep -Eq '^name: [a-z0-9-]+$' "$skill" ||
    fail "$skill: invalid or missing name"
  grep -Eq '^description: .+' "$skill" ||
    fail "$skill: missing description"

  if grep -Eq 'specs/|Specification|Atomic Feature' "$skill"; then
    grep -q 'SPEC_ID' "$skill" ||
      fail "$skill: specification skill does not require SPEC_ID"
  fi
done

generators=$(grep -El 'Allowed Write|Allowed Writes|write|Write' \
  "$root"/*/SKILL.md 2>/dev/null || true)

for skill in $generators; do
  grep -q 'explicit-user-input' "$skill" ||
    fail "$skill: missing explicit identifier provenance"
  grep -Eq 'branch|branches' "$skill" ||
    fail "$skill: missing branch-derived identifier rejection"
  grep -Eq 'session|sessions' "$skill" ||
    fail "$skill: missing session-derived identifier rejection"
  grep -Eq 'counter|allocation' "$skill" ||
    fail "$skill: missing repository allocation rejection"
done

violations=/tmp/kitlogger-skill-violations.txt
: >"$violations"

for skill in "$root"/*/SKILL.md; do
  case "$skill" in
    */spec-constitution-guardian/SKILL.md) continue ;;
  esac

  grep -niE \
    'speckit\.specify|/specify\b|specs/\{number\}|features/<FEATURE_ID>|Feature Specification:' \
    "$skill" >>"$violations" 2>/dev/null || true
done

if [ -s "$violations" ]; then
  cat "$violations" >&2
  fail "legacy specification workflow reference found"
fi

[ "$failed" -eq 0 ] || exit 1
printf 'Constitution v3.1 skill validation: PASS\n'
