#!/bin/sh
set -eu

REPO="jiusanzhou/usehid"
BINARY="usehid"
INSTALL_DIR="${USEHID_INSTALL_DIR:-/usr/local/bin}"

get_arch() {
  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    arm64|aarch64) echo "arm64" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac
}

get_os() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  case "$os" in
    linux)  echo "linux" ;;
    darwin) echo "macos" ;;
    *) echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac
}

get_latest_version() {
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/'
  elif command -v wget > /dev/null 2>&1; then
    wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/'
  else
    echo "curl or wget is required" >&2; exit 1
  fi
}

download() {
  url="$1"
  output="$2"
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget > /dev/null 2>&1; then
    wget -qO "$output" "$url"
  fi
}

main() {
  os=$(get_os)
  arch=$(get_arch)
  version="${USEHID_VERSION:-$(get_latest_version)}"

  if [ -z "$version" ]; then
    echo "Failed to determine latest version" >&2
    exit 1
  fi

  artifact="${BINARY}-${os}-${arch}"
  url="https://github.com/${REPO}/releases/download/v${version}/${artifact}"

  echo "Installing usehid v${version} (${os}/${arch})..."

  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  download "$url" "$tmpdir/$BINARY"
  chmod +x "$tmpdir/$BINARY"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
  else
    echo "Need sudo to install to $INSTALL_DIR"
    sudo mv "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
  fi

  echo "Installed usehid to $INSTALL_DIR/$BINARY"
  echo "Run 'usehid --help' to get started"
}

main
