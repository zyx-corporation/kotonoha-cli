#!/usr/bin/env bash
# Set legacy default principal role (M6 one-role-per-principal in shared DB).
# Usage: ./scripts/m6_legacy_rbac.sh [agent_runner|reviewer|owner]
set -euo pipefail

ROLE="${1:-agent_runner}"
LEGACY_PROJECT='00000000-0000-4000-8000-000000000002'
LEGACY_PRINCIPAL='00000000-0000-4000-8000-000000000001'

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "error: DATABASE_URL is required" >&2
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "error: psql is required for M6 RBAC bootstrap" >&2
  exit 1
fi

case "$ROLE" in
  agent_runner | reviewer | owner) ;;
  *)
    echo "error: unknown role: $ROLE" >&2
    exit 1
    ;;
esac

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -c "
  UPDATE project_members
  SET role = '${ROLE}'
  WHERE project_id = '${LEGACY_PROJECT}'::uuid
    AND principal_id = '${LEGACY_PRINCIPAL}'::uuid;
"

export KOTONOHA_PRINCIPAL_ID="${KOTONOHA_PRINCIPAL_ID:-$LEGACY_PRINCIPAL}"
export KOTONOHA_PROJECT_ID="${KOTONOHA_PROJECT_ID:-$LEGACY_PROJECT}"
