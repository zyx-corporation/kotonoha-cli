#!/usr/bin/env bash
# Install the official Kotonoha CLI (`kotonoha`) — curl-friendly entrypoint.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/zyx-corporation/kotonoha-cli/main/scripts/install.sh | bash
#   curl -fsSL ... | bash -s -- --version v0.2.9
#
# Environment:
#   KOTONOHA_VERSION      Release tag (default: latest GitHub release)
#   KOTONOHA_INSTALL_DIR  Prefix (default: $HOME/.local)
#   KOTONOHA_INSTALL_METHOD  auto | binary | cargo (default: auto)

set -euo pipefail

REPO="zyx-corporation/kotonoha-cli"
GIT_URL="https://github.com/${REPO}.git"
BINARY_NAME="kotonoha"
CRATE_NAME="kotonoha-cli"
INSTALL_DIR="${KOTONOHA_INSTALL_DIR:-${HOME}/.local}"
BIN_DIR="${INSTALL_DIR}/bin"
METHOD="${KOTONOHA_INSTALL_METHOD:-auto}"

log() { printf '==> %s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

detect_asset() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "${os}" in
    darwin) os="macos" ;;
    linux) os="linux" ;;
    *) die "unsupported OS: ${os} (use KOTONOHA_INSTALL_METHOD=cargo on supported hosts with Rust)" ;;
  esac
  case "${arch}" in
    x86_64 | amd64) arch="amd64" ;;
    arm64 | aarch64) arch="arm64" ;;
    *) die "unsupported architecture: ${arch}" ;;
  esac
  printf '%s-%s' "${os}" "${arch}"
}

resolve_version() {
  if [[ -n "${KOTONOHA_VERSION:-}" ]]; then
    printf '%s\n' "${KOTONOHA_VERSION}"
    return
  fi
  need_cmd curl
  local tag
  tag="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1)"
  [[ -n "${tag}" ]] || die "could not resolve latest release tag; set KOTONOHA_VERSION"
  printf '%s\n' "${tag}"
}

install_binary() {
  local version="$1"
  local asset="$2"
  need_cmd curl
  need_cmd tar
  local base="https://github.com/${REPO}/releases/download/${version}"
  local archive="${BINARY_NAME}-${version}-${asset}.tar.gz"
  local url="${base}/${archive}"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp}"' RETURN
  log "downloading ${url}"
  if ! curl -fsSL "${url}" -o "${tmp}/${archive}"; then
    return 1
  fi
  tar -xzf "${tmp}/${archive}" -C "${tmp}"
  mkdir -p "${BIN_DIR}"
  if [[ -f "${tmp}/${BINARY_NAME}" ]]; then
    install -m 0755 "${tmp}/${BINARY_NAME}" "${BIN_DIR}/${BINARY_NAME}"
  elif [[ -f "${tmp}/bin/${BINARY_NAME}" ]]; then
    install -m 0755 "${tmp}/bin/${BINARY_NAME}" "${BIN_DIR}/${BINARY_NAME}"
  else
    die "archive did not contain ${BINARY_NAME}"
  fi
  return 0
}

install_cargo() {
  local version="$1"
  need_cmd cargo
  mkdir -p "${INSTALL_DIR}"
  log "installing ${CRATE_NAME} ${version} via cargo (this may take several minutes)"
  cargo install "${CRATE_NAME}" \
    --git "${GIT_URL}" \
    --tag "${version}" \
    --locked \
    --root "${INSTALL_DIR}" \
    --force
}

verify_install() {
  export PATH="${BIN_DIR}:${PATH}"
  need_cmd "${BINARY_NAME}"
  log "installed: $("${BINARY_NAME}" version | head -n1)"
  "${BINARY_NAME}" version >/dev/null
}

print_path_hint() {
  cat <<EOF

Kotonoha CLI installed to: ${BIN_DIR}/${BINARY_NAME}

Add to your shell profile if needed:
  export PATH="${BIN_DIR}:\$PATH"

Verify:
  kotonoha version

EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)
        [[ $# -ge 2 ]] || die "--version requires a value"
        KOTONOHA_VERSION="$2"
        shift 2
        ;;
      --dir)
        [[ $# -ge 2 ]] || die "--dir requires a value"
        INSTALL_DIR="$2"
        BIN_DIR="${INSTALL_DIR}/bin"
        shift 2
        ;;
      --method)
        [[ $# -ge 2 ]] || die "--method requires auto|binary|cargo"
        METHOD="$2"
        shift 2
        ;;
      -h | --help)
        sed -n '1,12p' "$0"
        exit 0
        ;;
      *)
        die "unknown argument: $1"
        ;;
    esac
  done
}

main() {
  parse_args "$@"
  local version asset
  version="$(resolve_version)"
  asset="$(detect_asset)"
  log "target version=${version} platform=${asset} install_dir=${INSTALL_DIR} method=${METHOD}"

  case "${METHOD}" in
    auto | binary)
      if install_binary "${version}" "${asset}"; then
        verify_install
        print_path_hint
        exit 0
      fi
      if [[ "${METHOD}" == "binary" ]]; then
        die "binary install failed for ${version} (${asset}); no release asset or network error"
      fi
      warn "binary install unavailable; falling back to cargo"
      ;;
    cargo) ;;
    *) die "invalid KOTONOHA_INSTALL_METHOD: ${METHOD}" ;;
  esac

  install_cargo "${version}"
  verify_install
  print_path_hint
}

main "$@"
