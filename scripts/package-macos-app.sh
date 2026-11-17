#!/usr/bin/env bash
# Package the macOS PhairDLNA.app bundle and universal binary (PLT-014, MB-009).
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_DIR}"

OUTPUT_DIR="${REPO_DIR}/target/bundle"
APP_NAME="PhairDLNA.app"
APP_PATH="${OUTPUT_DIR}/${APP_NAME}"
CONTENTS="${APP_PATH}/Contents"
MACOS="${CONTENTS}/MacOS"
RESOURCES="${CONTENTS}/Resources"

BUILD_UNIVERSAL="${BUILD_UNIVERSAL:-1}"

echo "=== Packaging macOS Bundle for PhairDLNA ==="

# Build aarch64 binary
echo "Building aarch64-apple-darwin..."
PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --target aarch64-apple-darwin --bin phairdlna-mac

if [[ "${BUILD_UNIVERSAL}" == "1" ]]; then
    echo "Building x86_64-apple-darwin for universal binary (PLT-014)..."
    PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --target x86_64-apple-darwin --bin phairdlna-mac
fi

# Prepare .app directory structure
rm -rf "${APP_PATH}"
mkdir -p "${MACOS}" "${RESOURCES}"

if [[ "${BUILD_UNIVERSAL}" == "1" ]]; then
    echo "Creating Universal Binary (lipo)..."
    lipo -create -output "${MACOS}/phairdlna-mac" \
        "${REPO_DIR}/target/aarch64-apple-darwin/release/phairdlna-mac" \
        "${REPO_DIR}/target/x86_64-apple-darwin/release/phairdlna-mac"
else
    cp "${REPO_DIR}/target/aarch64-apple-darwin/release/phairdlna-mac" "${MACOS}/phairdlna-mac"
fi
chmod +x "${MACOS}/phairdlna-mac"

# Copy bundle resources
cp "${REPO_DIR}/crates/phairdlna-mac/resources/Info.plist" "${CONTENTS}/Info.plist"
cp "${REPO_DIR}/crates/phairdlna-mac/resources/en-US.strings" "${RESOURCES}/en.lproj/Localizable.strings" 2>/dev/null || \
    (mkdir -p "${RESOURCES}/en.lproj" && cp "${REPO_DIR}/crates/phairdlna-mac/resources/en-US.strings" "${RESOURCES}/en.lproj/Localizable.strings")
cp "${REPO_DIR}/crates/phairdlna-mac/resources/en-GB.strings" "${RESOURCES}/en-GB.lproj/Localizable.strings" 2>/dev/null || \
    (mkdir -p "${RESOURCES}/en-GB.lproj" && cp "${REPO_DIR}/crates/phairdlna-mac/resources/en-GB.strings" "${RESOURCES}/en-GB.lproj/Localizable.strings")

echo "APPL????" > "${CONTENTS}/PkgInfo"

echo "=== Verifying Bundle Structure ==="
file "${MACOS}/phairdlna-mac"
lipo -info "${MACOS}/phairdlna-mac" || true
ls -la "${CONTENTS}"
echo "SUCCESS: Bundle packaged at ${APP_PATH}"
