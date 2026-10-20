#!/usr/bin/env bash
# Install the prebuilt AHEM MNN fork libraries (GitHub release ahem-mnn-android-v1)
# into the app's jniLibs, refusing anything whose hash does not match.
# Used by build.sh when the fork's local build tree is not present (any fresh clone).
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
DEST=${1:-"$ROOT/src/Fintech_Wallet_Agent/fusion-core-android/app/src/main/jniLibs/arm64-v8a"}
URL=https://github.com/AdribMahmud101/fusion_wallet/releases/download/ahem-mnn-android-v1/ahem-mnn-android-v1.tar.gz
ARCHIVE_SHA256=43c8de1dcf1896e2f9f7be3643bb5ef3d922135095e2a39114b82800dcd9edc1
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
curl -fsSL "$URL" -o "$TMP/a.tgz"
echo "$ARCHIVE_SHA256  $TMP/a.tgz" | sha256sum -c --quiet
tar -C "$TMP" -xzf "$TMP/a.tgz"
(cd "$TMP/ahem-mnn-android-v1" && sha256sum -c --quiet SHA256SUMS)
mkdir -p "$DEST"
cp "$TMP"/ahem-mnn-android-v1/lib/arm64-v8a/*.so "$DEST/"
echo "Installed verified AHEM MNN libraries into $DEST"
