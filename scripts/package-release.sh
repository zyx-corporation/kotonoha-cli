#!/usr/bin/env bash
# Package a built `kotonoha` binary into an install.sh-compatible tarball.
# Output is written to the current directory and must NOT be committed to git.
#
# Usage:
#   cargo build --release --target <triple>
#   ./scripts/package-release.sh v0.2.9 linux-amd64 target/<triple>/release/kotonoha
#
# CI and release.yml call this script after `cargo build --release`.

set -euo pipefail

tag="${1:-}"
asset="${2:-}"
binary_path="${3:-}"

if [[ -z "${tag}" || -z "${asset}" || -z "${binary_path}" ]]; then
  echo "usage: $0 <tag> <asset> <path-to-kotonoha-binary>" >&2
  echo "  example: $0 v0.2.9 linux-amd64 target/x86_64-unknown-linux-gnu/release/kotonoha" >&2
  exit 1
fi

if [[ ! -f "${binary_path}" ]]; then
  echo "error: binary not found: ${binary_path}" >&2
  exit 1
fi

archive="kotonoha-${tag}-${asset}.tar.gz"
staging="$(mktemp -d)"
trap 'rm -rf "${staging}"' EXIT

cp "${binary_path}" "${staging}/kotonoha"
chmod 0755 "${staging}/kotonoha"

tar -czf "${archive}" -C "${staging}" kotonoha
echo "created ${archive} ($(wc -c < "${archive}") bytes)"
