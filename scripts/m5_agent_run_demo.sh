#!/usr/bin/env bash
# M5 acceptance demo: context pack → AgentRun → agent delta → RDE attach → capability deny.
# Usage:
#   ./scripts/m5_agent_run_demo.sh [PATH_TO_KOTONOHA_BINARY]
#
# Requires:
#   DATABASE_URL — PostgreSQL (M1 + M5 migrations via `kotonoha db migrate`)
#   Git repository cwd
#
# See: kotonoha-management docs/31_m5_agent_run_integration_spec_draft.md §9
#      docs/chatgpt-app/04_mcp_tools_and_ux.md §6
# Pattern A: docs/26_rde_llm_connection_design_draft.md §4.1
set -euo pipefail

KO="${1:-${KOTONOHA_BIN:-./target/release/kotonoha}}"
if [[ ! -f "$KO" ]]; then
  echo "error: kotonoha binary not found: $KO" >&2
  echo "  build with: cargo build --release  (or pass path as first argument)" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "error: DATABASE_URL is required for M5 demo" >&2
  exit 1
fi

echo "== M5 agent-run demo (binary: $KO) =="

"$KO" db migrate

DEMO_FILE="${M5_DEMO_FILE:-docs/m5_demo_scratch.md}"
mkdir -p "$(dirname "$DEMO_FILE")"
echo "# M5 demo $(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$DEMO_FILE"

echo "--- Step 1: context export (no DATABASE_URL) ---"
CTX=$(env -u DATABASE_URL "$KO" context export "$DEMO_FILE")
echo "$CTX" | head -n 20
echo "$CTX" | grep -q '"kotonoha.context_pack.v0.1"' || {
  echo "error: context export missing kotonoha.context_pack.v0.1" >&2
  exit 1
}

echo "--- Step 2: rde validate (mock LLM / emit skeleton) ---"
"$KO" rde emit | "$KO" rde validate --strict

echo "--- Step 3: agent record start ---"
RUN_ID=$("$KO" agent record start --agent-kind m5-demo --external-ref "demo-$(date +%s)")
echo "agent_run_id: $RUN_ID"

OBS="${M5_DEMO_OBSERVATION:-/tmp/m5_observation.json}"
echo '{"preserved":["intent"],"intended_change":"M5 demo observation"}' >"$OBS"

echo "--- Step 4: agent delta create (links agent_run_id) ---"
DELTA=$("$KO" agent delta create "$DEMO_FILE" --agent-run-id "$RUN_ID" --observation "$OBS")
echo "meaning_delta_id: $DELTA"

echo "--- Step 5: rde attach (source_kind=llm) ---"
ASSESSMENT=$("$KO" rde emit | "$KO" rde attach --delta-id "$DELTA" --source-kind llm --strict)
echo "rde_assessment_id: $ASSESSMENT"

echo "--- Step 6: agent record complete ---"
"$KO" agent record complete --run-id "$RUN_ID"

echo "--- Step 7: agent-channel review approve (expect deny, exit 2) ---"
set +e
"$KO" review approve \
  --delta-id "$DELTA" \
  --assessment-id "$ASSESSMENT" \
  --agent-run-id "$RUN_ID" \
  --decided-by "agent-bot" 2>&1 | tee /tmp/m5_agent_deny.log
DENY_CODE=${PIPESTATUS[0]}
set -e
if [[ "$DENY_CODE" -ne 2 ]]; then
  echo "error: expected exit 2 for agent review approve, got $DENY_CODE" >&2
  exit 1
fi
grep -q "denied_actions" /tmp/m5_agent_deny.log || {
  echo "error: stderr should mention denied_actions" >&2
  exit 1
}

echo "--- Step 8: human review approve (no agent context) ---"
DECISION=$("$KO" review approve \
  --delta-id "$DELTA" \
  --assessment-id "$ASSESSMENT" \
  --decided-by "human-reviewer")
echo "review_decision_id: $DECISION"

echo "export m2 snapshot:"
"$KO" export --delta-id "$DELTA" --format m2 | head -n 35

echo "== M5 demo complete =="
