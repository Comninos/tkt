#!/bin/sh
# Install prebuilt tkt from GitHub Releases into ~/.local/bin (or $BIN_DIR).
# Falls back to `cargo install` if no matching release asset exists.
set -eu

REPO="${TKT_REPO:-Comninos/tkt}"
BIN_DIR="${BIN_DIR:-${HOME}/.local/bin}"
BIN_NAME="tkt"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: '$1' is required" >&2
    exit 1
  fi
}

detect_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)
      echo "error: unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  case "$os" in
    linux*) target="${arch}-unknown-linux-gnu" ;;
    darwin*) target="${arch}-apple-darwin" ;;
    mingw*|msys*|cygwin*)
      echo "error: use install.ps1 on Windows" >&2
      exit 1
      ;;
    *)
      echo "error: unsupported OS: $os" >&2
      exit 1
      ;;
  esac

  printf '%s\n' "$target"
}

install_from_cargo() {
  need_cmd cargo
  echo "installing from source with cargo..."
  cargo install --git "https://github.com/${REPO}" --locked
  echo "installed: $(command -v "$BIN_NAME" || echo tkt)"
}

download() {
  url=$1
  dest=$2
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    echo "error: need curl or wget" >&2
    exit 1
  fi
}

need_cmd tar
need_cmd uname
need_cmd mktemp

target=$(detect_target)
asset="tkt-${target}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

echo "downloading ${asset}..."
if ! download "$url" "${tmpdir}/${asset}"; then
  echo "no prebuilt binary for ${target}; falling back to cargo" >&2
  install_from_cargo
  exit 0
fi

tar -xzf "${tmpdir}/${asset}" -C "$tmpdir"
mkdir -p "$BIN_DIR"

if [ ! -f "${tmpdir}/${BIN_NAME}" ]; then
  echo "error: archive did not contain ${BIN_NAME}" >&2
  exit 1
fi

install -m 755 "${tmpdir}/${BIN_NAME}" "${BIN_DIR}/${BIN_NAME}"
echo "installed: ${BIN_DIR}/${BIN_NAME}"

case ":${PATH}:" in
  *":${BIN_DIR}:"*) ;;
  *)
    echo "note: add ${BIN_DIR} to your PATH if needed" >&2
    ;;
esac
