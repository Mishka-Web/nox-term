#!/usr/bin/env sh
set -eu

if [ -z "${NOX_GITHUB_REPOSITORY:-}" ] && command -v git >/dev/null 2>&1; then
  REMOTE="$(git config --get remote.origin.url 2>/dev/null || true)"
  case "$REMOTE" in
    *github.com:*) NOX_GITHUB_REPOSITORY="${REMOTE#*github.com:}" ;;
    *github.com/*) NOX_GITHUB_REPOSITORY="${REMOTE#*github.com/}" ;;
  esac
  NOX_GITHUB_REPOSITORY="${NOX_GITHUB_REPOSITORY%.git}"
fi

if [ -n "${NOX_GITHUB_REPOSITORY:-}" ]; then
  export NOX_GITHUB_REPOSITORY
  echo "Self-update repository: $NOX_GITHUB_REPOSITORY"
else
  echo "Warning: repository is unknown; 'nox update' will require NOX_GITHUB_REPOSITORY=owner/repo" >&2
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS:$ARCH" in
  Linux:x86_64) TARGET="x86_64-unknown-linux-musl"; PACKAGE="nox-linux-x64" ;;
  Linux:aarch64|Linux:arm64) TARGET="aarch64-unknown-linux-musl"; PACKAGE="nox-linux-arm64" ;;
  Darwin:x86_64) TARGET="x86_64-apple-darwin"; PACKAGE="nox-macos-x64" ;;
  Darwin:arm64) TARGET="aarch64-apple-darwin"; PACKAGE="nox-macos-arm64" ;;
  *) echo "Unsupported host: $OS $ARCH" >&2; exit 1 ;;
esac

rustup target add "$TARGET"

if [ "$OS" = "Linux" ] && ! command -v musl-gcc >/dev/null 2>&1; then
  echo "musl-gcc is required for the portable Linux build." >&2
  echo "Ubuntu/Debian: sudo apt install musl-tools" >&2
  exit 1
fi

cargo test --release --target "$TARGET"
cargo build --release --target "$TARGET"

mkdir -p "dist/$PACKAGE"
cp "target/$TARGET/release/nox" "dist/$PACKAGE/nox"
cp README.md PORTABLE.md LICENSE "dist/$PACKAGE/"
chmod +x "dist/$PACKAGE/nox"

echo "Portable build: dist/$PACKAGE/nox"
