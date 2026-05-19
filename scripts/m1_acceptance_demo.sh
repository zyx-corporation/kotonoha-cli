#!/usr/bin/env bash
# M1 acceptance-style demo: MeaningDelta → RDE attach → review → export.
# Usage:
#   ./scripts/m1_acceptance_demo.sh [PATH_TO_KOTONOHA_BINARY]
#
# Requires:
#   DATABASE_URL — PostgreSQL with M1 schema (`kotonoha db migrate`)
#   Git repository cwd (or run from repo root)
#
set -euo pipefail

KO="${1:-${KOTONOHA_BIN:-./target/release/kotonoha}}"
if [[ ! -f "$KO" ]]; then
  echo "error: kotonoha binary not found: $KO" >&2
  echo "  build with: cargo build --release  (or pass path as first argument)" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "error: DATABASE_URL is required for M1 demo" >&2
  exit 1
fi

echo "== M1 acceptance demo (binary: $KO) =="

"$KO" db migrate

DEMO_FILE="${M1_DEMO_FILE:-docs/m1_demo_scratch.md}"
mkdir -p "$(dirname "$DEMO_FILE")"
echo "# M1 demo $(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$DEMO_FILE"

DELTA=$("$KO" delta create "$DEMO_FILE")
echo "meaning_delta_id: $DELTA"

ASSESSMENT=$("$KO" rde emit | "$KO" rde attach --delta-id "$DELTA")
echo "rde_assessment_id: $ASSESSMENT"

DECISION=$("$KO" review approve --delta-id "$DELTA" --assessment-id "$ASSESSMENT" --decided-by "m1-demo")
echo "review_decision_id: $DECISION"

COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "(no commits yet)")
echo "export by delta-id:"
"$KO" export --delta-id "$DELTA" | head -n 20

echo "export by git-commit ($COMMIT):"
"$KO" export --git-commit "$COMMIT" | head -n 25

echo "== M1 demo complete =="
