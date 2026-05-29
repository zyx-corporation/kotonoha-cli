#!/usr/bin/env bash
# M7 smoke: project-scoped m6 export isolation (wraps cargo test).
# Usage: ./scripts/m7_export_smoke.sh
# Requires: DATABASE_URL, git (same as CI m6_integration)
set -euo pipefail

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "skip m7_export_smoke: DATABASE_URL not set" >&2
  exit 0
fi

echo "== M7 export smoke (m6_integration) =="
cargo test --test m6_integration m6_export_isolates_two_projects_on_same_commit -- --nocapture
echo "== M7 export smoke OK =="
