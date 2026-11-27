#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-v0.1.0-alpha.4}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_DIR}"

ASSETS_DIR="${REPO_DIR}/target/release-assets"
mkdir -p "${ASSETS_DIR}"

echo "=== 1. Packaging macOS App ==="
./scripts/package-macos-app.sh

APP_PATH="${REPO_DIR}/target/bundle/PhairDLNA.app"
DMG_PATH="${ASSETS_DIR}/PhairDLNA-${VERSION}-macOS-universal.dmg"
DMG_TMP="${REPO_DIR}/target/dmg_staging"

echo "=== 2. Creating DMG Image ==="
rm -rf "${DMG_TMP}" "${DMG_PATH}"
mkdir -p "${DMG_TMP}"
cp -R "${APP_PATH}" "${DMG_TMP}/"
ln -s /Applications "${DMG_TMP}/Applications"

hdiutil create -volname "PhairDLNA" -srcfolder "${DMG_TMP}" -ov -format UDZO "${DMG_PATH}"
rm -rf "${DMG_TMP}"

echo "=== 3. Packaging CLI Universal Binary ==="
PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --target aarch64-apple-darwin --bin phairdlna-cli
PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --target x86_64-apple-darwin --bin phairdlna-cli

CLI_STAGING="${REPO_DIR}/target/cli_staging"
rm -rf "${CLI_STAGING}"
mkdir -p "${CLI_STAGING}"

lipo -create -output "${CLI_STAGING}/phairdlna" \
    "${REPO_DIR}/target/aarch64-apple-darwin/release/phairdlna-cli" \
    "${REPO_DIR}/target/x86_64-apple-darwin/release/phairdlna-cli"
chmod +x "${CLI_STAGING}/phairdlna"
cp LICENSE "${CLI_STAGING}/"
cp README.md "${CLI_STAGING}/"

CLI_TAR="${ASSETS_DIR}/phairdlna-${VERSION}-macOS-universal-cli.tar.gz"
tar -czf "${CLI_TAR}" -C "${CLI_STAGING}" phairdlna LICENSE README.md
rm -rf "${CLI_STAGING}"

echo "=== 4. Packaging Linux Debian Package (M3.3, PLT-016) ==="
./scripts/package-linux-deb.sh "${VERSION}"
DEB_PKG="${ASSETS_DIR}/phairdlna_${VERSION#v}_amd64.deb"

echo "=== 5. Generating SHA256 Checksums ==="
cd "${ASSETS_DIR}"
shasum -a 256 "$(basename "${DMG_PATH}")" "$(basename "${CLI_TAR}")" "$(basename "${DEB_PKG}")" > "SHA256SUMS-${VERSION}.txt"

echo "=== SUCCESS: Assets built ==="
ls -la "${ASSETS_DIR}"/*"${VERSION}"*
