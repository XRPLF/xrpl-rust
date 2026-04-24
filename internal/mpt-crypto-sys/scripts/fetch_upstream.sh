#!/usr/bin/env bash
# Refresh vendored mpt-crypto binaries from an upstream GitHub release.
#
# Run when bumping MPT_CRYPTO_VERSION in build.rs. After this script, also:
#   1. Update MPT_CRYPTO_VERSION  in build.rs to the new tag.
#   2. Update BUNDLE_SHA256       in build.rs to the value this script prints.
#   3. scripts/regenerate_bindings.sh   — refresh src/bindings.rs.
#   4. Commit: vendor/include, vendor/lib, src/bindings.rs, build.rs.
#
# Requires: curl, tar, shasum.
#
# Usage:
#   scripts/fetch_upstream.sh <tag>
#   scripts/fetch_upstream.sh 0.3.0-rc2

set -euo pipefail

TAG="${1:-}"
if [ -z "$TAG" ]; then
  echo "Usage: $0 <tag>     (e.g. 0.3.0-rc2)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR="$CRATE_ROOT/vendor"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

URL="https://github.com/XRPLF/mpt-crypto/releases/download/${TAG}/mpt-crypto-natives-${TAG}.tar.gz"
BUNDLE="$TMP/bundle.tar.gz"

echo "==> Downloading ${URL}"
curl -sSL -o "$BUNDLE" "$URL"

echo "==> Verifying download"
BUNDLE_SHA256="$(shasum -a 256 "$BUNDLE" | awk '{print $1}')"
echo "    SHA-256: $BUNDLE_SHA256"

echo "==> Extracting to $VENDOR/"
mkdir -p "$VENDOR/include/utility" "$VENDOR/lib"
tar -xzf "$BUNDLE" -C "$TMP"

cp "$TMP/include/secp256k1_mpt.h"       "$VENDOR/include/"
cp "$TMP/include/utility/mpt_utility.h" "$VENDOR/include/utility/"

# Map upstream platform names → Rust target triples
for PAIR in \
  "darwin-aarch64:aarch64-apple-darwin" \
  "darwin-x86-64:x86_64-apple-darwin" \
  "linux-aarch64:aarch64-unknown-linux-gnu" \
  "linux-x86-64:x86_64-unknown-linux-gnu" \
  "linux-s390x:s390x-unknown-linux-gnu" \
  "win32-x86-64:x86_64-pc-windows-msvc"
do
  UP="${PAIR%%:*}"
  TARGET="${PAIR##*:}"
  if [ -d "$TMP/$UP" ]; then
    mkdir -p "$VENDOR/lib/$TARGET"
    cp "$TMP/$UP/"* "$VENDOR/lib/$TARGET/"
    echo "    $UP → $TARGET"
  else
    echo "    $UP missing in bundle (skipping)"
  fi
done

echo ""
echo "==> Done."
echo ""
echo "Next: update build.rs constants and run scripts/regenerate_bindings.sh"
echo "      MPT_CRYPTO_VERSION = \"$TAG\""
echo "      BUNDLE_SHA256      = \"$BUNDLE_SHA256\""
