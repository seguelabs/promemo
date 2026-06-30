#!/usr/bin/env sh
set -eu

REPO="${PROMEMO_REPO:-seguelabsai/promemo}"
TAG="${PROMEMO_VERSION:-latest}"
INSTALL_DIR="${PROMEMO_INSTALL_DIR:-$HOME/.local/bin}"

fail() {
  printf '%s\n' "promemo install: $*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

need curl
need tar

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$os:$arch" in
  darwin:arm64|darwin:aarch64)
    asset="promemo-aarch64-apple-darwin.tar.gz"
    binary="promemo"
    ;;
  linux:x86_64|linux:amd64)
    asset="promemo-x86_64-unknown-linux-gnu.tar.gz"
    binary="promemo"
    ;;
  mingw*:x86_64|msys*:x86_64|cygwin*:x86_64)
    asset="promemo-x86_64-pc-windows-msvc.zip"
    binary="promemo.exe"
    ;;
  *)
    fail "unsupported platform: $os $arch"
    ;;
esac

if [ "$TAG" = "latest" ]; then
  base_url="https://github.com/$REPO/releases/latest/download"
else
  case "$TAG" in
    v*) ;;
    *) TAG="v$TAG" ;;
  esac
  base_url="https://github.com/$REPO/releases/download/$TAG"
fi

tmp="${TMPDIR:-/tmp}/promemo-install-$$"
mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT INT TERM

archive="$tmp/$asset"
checksums="$tmp/SHA256SUMS"
package_dir="${asset%.tar.gz}"
package_dir="${package_dir%.zip}"

curl -fsSL "$base_url/$asset" -o "$archive"
curl -fsSL "$base_url/SHA256SUMS" -o "$checksums"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmp" && grep "  $asset\$" SHA256SUMS | sha256sum -c -)
elif command -v shasum >/dev/null 2>&1; then
  expected="$(grep "  $asset\$" "$checksums" | awk '{print $1}')"
  actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
  [ "$expected" = "$actual" ] || fail "checksum mismatch for $asset"
else
  printf '%s\n' "promemo install: warning: no SHA256 tool found; skipping checksum verification" >&2
fi

mkdir -p "$INSTALL_DIR"

case "$asset" in
  *.zip)
    need unzip
    unzip -q "$archive" -d "$tmp/extract"
    cp "$tmp/extract/$package_dir/$binary" "$INSTALL_DIR/$binary"
    chmod +x "$INSTALL_DIR/$binary"
    ;;
  *.tar.gz)
    mkdir -p "$tmp/extract"
    tar -xzf "$archive" -C "$tmp/extract"
    cp "$tmp/extract/$package_dir/$binary" "$INSTALL_DIR/promemo"
    chmod +x "$INSTALL_DIR/promemo"
    ;;
esac

printf '%s\n' "Installed Promemo to $INSTALL_DIR"
printf '%s\n' "Run: $INSTALL_DIR/$binary --version"
