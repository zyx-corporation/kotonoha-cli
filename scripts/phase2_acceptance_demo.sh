#!/usr/bin/env bash
# Phase 2 acceptance-style checks for the `kotonoha` CLI (see docs/cli-definition.md).
# Usage:
#   ./scripts/phase2_acceptance_demo.sh [PATH_TO_KOTONOHA_BINARY]
#
# Env:
#   DATABASE_URL — if set, runs db migrate + interchange store (recommended for full gate coverage).
#
set -euo pipefail

KO="${1:-${KOTONOHA_BIN:-./target/release/kotonoha}}"
if [[ ! -f "$KO" ]]; then
  echo "error: kotonoha binary not found: $KO" >&2
  echo "  build with: cargo build --release  (or pass path as first argument)" >&2
  exit 1
fi

echo "== Phase 2 acceptance demo (binary: $KO) =="

echo "[A] kotonoha version"
"$KO" version

echo "[B] RDE emit | rde validate --strict"
"$KO" rde emit | "$KO" rde validate --strict

echo "[C] interchange emit | interchange validate --strict"
"$KO" interchange emit | "$KO" interchange validate --strict

echo "[E] invalid JSON -> exit 2 (contract)"
set +e
code=0
echo '{}' | "$KO" rde validate --strict >/dev/null 2>&1
code=$?
set -euo pipefail
if [[ "$code" -ne 2 ]]; then
  echo "error: expected rde validate exit code 2 for '{}', got $code" >&2
  exit 1
fi

if [[ -n "${DATABASE_URL:-}" ]]; then
  echo "[D] DATABASE_URL set -> db migrate + interchange store"
  "$KO" db migrate
  out=$("$KO" interchange emit | "$KO" interchange store --strict)
  if [[ -z "$out" ]]; then
    echo "error: interchange store produced empty stdout (expected UUID)" >&2
    exit 1
  fi
  echo "    stored interchange_documents id: $out"
else
  echo "[D] skip persistence (unset DATABASE_URL)"
fi

echo "== Phase 2 acceptance demo: OK =="
