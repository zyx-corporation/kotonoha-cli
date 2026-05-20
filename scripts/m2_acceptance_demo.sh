#!/usr/bin/env bash
# M2 acceptance demo: M1 path + RDE meta attach + m2 export.
# Usage:
#   ./scripts/m2_acceptance_demo.sh [PATH_TO_KOTONOHA_BINARY]
#
# Requires:
#   DATABASE_URL — PostgreSQL (`kotonoha db migrate` applies M1 + M2)
#   Git repository cwd
#
# See: kotonoha-management docs/28_m2_rde_record_integration_spec_draft.md
# LLM pattern A: docs/26_rde_llm_connection_design_draft.md
set -euo pipefail

KO="${1:-${KOTONOHA_BIN:-./target/release/kotonoha}}"
if [[ ! -f "$KO" ]]; then
  echo "error: kotonoha binary not found: $KO" >&2
  echo "  build with: cargo build --release  (or pass path as first argument)" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "error: DATABASE_URL is required for M2 demo" >&2
  exit 1
fi

echo "== M2 acceptance demo (binary: $KO) =="

"$KO" db migrate

DEMO_FILE="${M2_DEMO_FILE:-docs/m2_demo_scratch.md}"
mkdir -p "$(dirname "$DEMO_FILE")"
echo "# M2 demo $(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$DEMO_FILE"

OBS="${M2_DEMO_OBSERVATION:-/tmp/m2_observation.json}"
echo '{"preserved":["intent"],"lost":[]}' >"$OBS"

DELTA=$("$KO" delta create "$DEMO_FILE" --observation "$OBS")
echo "meaning_delta_id: $DELTA"

ASSESSMENT=$("$KO" rde emit | "$KO" rde attach --delta-id "$DELTA" --source-kind cli)
echo "rde_assessment_id: $ASSESSMENT"

DECISION=$("$KO" review approve --delta-id "$DELTA" --assessment-id "$ASSESSMENT" --decided-by "m2-demo")
echo "review_decision_id: $DECISION"

COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "(no commits yet)")
echo "export m2 by delta-id:"
"$KO" export --delta-id "$DELTA" --format m2 | head -n 30

echo "export m2 by git-commit ($COMMIT):"
"$KO" export --git-commit "$COMMIT" --format m2 | head -n 35

echo "== M2 demo complete =="
