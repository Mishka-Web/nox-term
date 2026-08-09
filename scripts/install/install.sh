#!/usr/bin/env sh
set -eu

REPOSITORY="__NOX_REPOSITORY__"
BASE_URL="https://github.com/$REPOSITORY/releases/latest/download"
INSTALL_DIR="${NOX_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
fail() { printf 'NOX install error: %s\n' "$*" >&2; exit 1; }

command -v uname >/dev/null 2>&1 || fail "uname is required"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS:$ARCH" in
  Linux:x86_64|Linux:amd64) ASSET="nox-linux-x64" ;;
  Linux:aarch64|Linux:arm64) ASSET="nox-linux-arm64" ;;
  Darwin:x86_64|Darwin:amd64) ASSET="nox-macos-x64" ;;
  Darwin:aarch64|Darwin:arm64) ASSET="nox-macos-arm64" ;;
  *) fail "unsupported platform: $OS/$ARCH" ;;
esac

TMP_DIR="${TMPDIR:-/tmp}/nox-install-$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

fetch() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fL --retry 3 --connect-timeout 15 "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -O "$output" "$url"
  else
    fail "curl or wget is required"
  fi
}

say "Installing NOX from $REPOSITORY"
fetch "$BASE_URL/$ASSET" "$TMP_DIR/$ASSET"
fetch "$BASE_URL/SHA256SUMS" "$TMP_DIR/SHA256SUMS"

EXPECTED="$(awk -v file="$ASSET" '$2 == file || $2 == "*" file { print $1; exit }' "$TMP_DIR/SHA256SUMS")"
[ -n "$EXPECTED" ] || fail "checksum for $ASSET was not found"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL="$(sha256sum "$TMP_DIR/$ASSET" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL="$(shasum -a 256 "$TMP_DIR/$ASSET" | awk '{print $1}')"
else
  fail "sha256sum or shasum is required"
fi

[ "$EXPECTED" = "$ACTUAL" ] || fail "SHA-256 verification failed"
say "SHA-256 verified"

mkdir -p "$INSTALL_DIR"
chmod +x "$TMP_DIR/$ASSET"
mv "$TMP_DIR/$ASSET" "$INSTALL_DIR/nox"
chmod +x "$INSTALL_DIR/nox"

VERSION="$($INSTALL_DIR/nox --version 2>/dev/null || true)"
say "Installed ${VERSION:-NOX} to $INSTALL_DIR/nox"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    say "Run: nox"
    ;;
  *)
    say "Add NOX to PATH for future shells:"
    say "  export PATH=\"$INSTALL_DIR:\$PATH\""
    say "Or run now: $INSTALL_DIR/nox"
    ;;
esac
