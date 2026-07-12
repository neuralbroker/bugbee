#!/usr/bin/env bash
# Bugbee one-line installer
#   curl -fsSL https://github.com/neuralbroker/bugbee/releases/latest/download/get-bugbee.sh | bash
#
# Options (env):
#   BUGBEE_VERSION   Tag to install (default: newest GitHub release, including prereleases)
#   BUGBEE_INSTALL   Install directory (default: $HOME/.local/bin)
#   BUGBEE_NO_PATH   Set to 1 to skip PATH hint

set -euo pipefail

REPO="neuralbroker/bugbee"
API_BASE="https://api.github.com/repos/${REPO}"
DOWNLOAD_BASE="https://github.com/${REPO}/releases/download"
INSTALL_DIR="${BUGBEE_INSTALL:-${HOME}/.local/bin}"
VERSION="${BUGBEE_VERSION:-}"

say() { printf '==> %s\n' "$*"; }
err() { printf 'error: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

detect_target() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "${os}" in
    linux) os="unknown-linux-gnu" ;;
    darwin) os="apple-darwin" ;;
    mingw* | msys* | cygwin*)
      err "Windows detected. Download the .zip from https://github.com/${REPO}/releases or use WSL."
      ;;
    *) err "unsupported OS: ${os}" ;;
  esac

  case "${arch}" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *) err "unsupported architecture: ${arch}" ;;
  esac

  # GitHub Actions matrix only ships macOS arm64 + intel, Linux x86_64.
  if [[ "${os}" == "unknown-linux-gnu" && "${arch}" == "aarch64" ]]; then
    err "Linux arm64 binaries are not published yet. Build from source: cargo install --git https://github.com/${REPO} --locked --bin bugbee"
  fi

  printf '%s-%s' "${arch}" "${os}"
}

latest_tag() {
  # Prefer the newest non-draft release (includes prereleases for beta).
  local json
  json="$(curl -fsSL "${API_BASE}/releases?per_page=20")" || err "could not query GitHub releases"
  printf '%s' "${json}" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1
}

download() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 --retry-delay 1 -o "${dest}" "${url}"
  else
    wget -q -O "${dest}" "${url}"
  fi
}

verify_sha256() {
  local archive="$1" checksum_file="$2"
  if [[ ! -f "${checksum_file}" ]]; then
    say "no checksum file published; skipping verification"
    return 0
  fi
  local expected actual
  expected="$(awk '{print $1}' "${checksum_file}" | tr '[:upper:]' '[:lower:]' | head -n 1)"
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${archive}" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${archive}" | awk '{print $1}')"
  else
    say "sha256 tool not found; skipping verification"
    return 0
  fi
  [[ "${expected}" == "${actual}" ]] || err "checksum mismatch for ${archive}"
  say "checksum verified"
}

main() {
  need_cmd uname
  need_cmd tar
  need_cmd mktemp
  if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
    err "curl or wget is required"
  fi

  local target
  target="$(detect_target)"

  if [[ -z "${VERSION}" ]]; then
    say "resolving latest release from GitHub"
    VERSION="$(latest_tag)"
    [[ -n "${VERSION}" ]] || err "no GitHub releases found. Try: cargo install --git https://github.com/${REPO} --locked --bin bugbee"
  fi
  # Accept tags with or without leading v
  case "${VERSION}" in
    v*) ;;
    *) VERSION="v${VERSION}" ;;
  esac

  local asset="bugbee-${target}.tar.gz"
  local url="${DOWNLOAD_BASE}/${VERSION}/${asset}"
  # Keep the cleanup path global: EXIT traps run after `main` returns, so a
  # `local tmp` would be unbound under `set -u`.
  BUGBEE_TMP="$(mktemp -d)"
  trap 'rm -rf "${BUGBEE_TMP:-}"' EXIT

  say "installing Bugbee ${VERSION} (${target})"
  say "download ${url}"
  if ! download "${url}" "${BUGBEE_TMP}/${asset}"; then
    err "failed to download ${url}. Is the release published? Fallback: cargo install --git https://github.com/${REPO} --locked --bin bugbee"
  fi

  if download "${url}.sha256" "${BUGBEE_TMP}/${asset}.sha256" 2>/dev/null; then
    verify_sha256 "${BUGBEE_TMP}/${asset}" "${BUGBEE_TMP}/${asset}.sha256"
  else
    say "checksum not available; continuing"
  fi

  tar -xzf "${BUGBEE_TMP}/${asset}" -C "${BUGBEE_TMP}"
  [[ -f "${BUGBEE_TMP}/bugbee" ]] || err "archive missing bugbee binary"

  mkdir -p "${INSTALL_DIR}"
  install -m 755 "${BUGBEE_TMP}/bugbee" "${INSTALL_DIR}/bugbee"

  say "installed ${INSTALL_DIR}/bugbee"
  if ! command -v bugbee >/dev/null 2>&1; then
    if [[ "${BUGBEE_NO_PATH:-0}" != "1" ]]; then
      cat <<EOF

Add Bugbee to your PATH (add to ~/.bashrc or ~/.zshrc):

  export PATH="${INSTALL_DIR}:\$PATH"

Then open a new shell and run:

  bugbee --version
  bugbee init
  bugbee hunt
EOF
    fi
  else
    "${INSTALL_DIR}/bugbee" --version || true
    cat <<'EOF'

Next:

  cd /path/to/your/project
  bugbee init
  bugbee doctor
  bugbee hunt
  bugbee findings
EOF
  fi
}

main "$@"
