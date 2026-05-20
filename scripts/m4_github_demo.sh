#!/usr/bin/env bash
# M4 GitHub integration smoke (local PostgreSQL).
# Skips gh-dependent steps when `gh` is not authenticated.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is not set — skip demo"
  exit 1
fi

KOTONOHA="${KOTONOHA:-target/release/kotonoha}"
if [[ ! -x "$KOTONOHA" ]]; then
  cargo build --release
  KOTONOHA="$ROOT/target/release/kotonoha"
fi

"$KOTONOHA" db migrate
"$KOTONOHA" github gh-status || echo "(gh-status failed — continuing DB-only steps)"

REPO_PATH="${REPO_PATH:-.}"
FILE="${FILE:-README.md}"
[[ -f "$FILE" ]] || FILE="Cargo.toml"

DELTA="$("$KOTONOHA" delta create "$FILE" --path "$REPO_PATH" | tail -1)"
echo "delta_id=$DELTA"

REPO_ID="$("$KOTONOHA" github link repo --path "$REPO_PATH" | tail -1)"
echo "repository_link_id=$REPO_ID"

"$KOTONOHA" github link issue --delta-id "$DELTA" --issue-number 1 --path "$REPO_PATH" || true

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  PR_NUM="${PR_NUM:-1}"
  "$KOTONOHA" github link pr --delta-id "$DELTA" --pr-number "$PR_NUM" --path "$REPO_PATH" || true
  "$KOTONOHA" github list-pr --pr-number "$PR_NUM" --path "$REPO_PATH" --json || true
  echo "--- pr-summary (en) ---"
  "$KOTONOHA" github pr-summary --pr-number "$PR_NUM" --delta-id "$DELTA" --locale en --path "$REPO_PATH" || true
else
  echo "gh not available — skipped PR steps"
fi

echo "M4 demo done"
