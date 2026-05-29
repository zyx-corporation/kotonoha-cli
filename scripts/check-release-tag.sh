#!/usr/bin/env bash
# Ensure git tag matches Cargo.toml version (e.g. tag v0.2.9 ↔ version "0.2.9").
set -euo pipefail

tag="${1:-}"
if [[ -z "${tag}" ]]; then
  echo "usage: $0 <tag>   (example: v0.2.9)" >&2
  exit 1
fi

if [[ ! "${tag}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "error: tag must look like vMAJOR.MINOR.PATCH (got: ${tag})" >&2
  exit 1
fi

cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
expected_tag="v${cargo_version}"

if [[ "${tag}" != "${expected_tag}" ]]; then
  echo "error: tag ${tag} does not match Cargo.toml version ${cargo_version} (expected ${expected_tag})" >&2
  exit 1
fi

echo "ok: tag ${tag} matches Cargo.toml version ${cargo_version}"
