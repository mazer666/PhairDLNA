#!/usr/bin/env bash
# Generates a standard Debian (.deb) package layout and archive for PhairDLNA (PLT-016, SUP-005, M3.3).
# Adheres strictly to native unconfined Linux packaging requirements (no snaps/flatpaks).
set -euo pipefail

VERSION="${1:-v0.1.0-alpha.8}"
VERSION_NUM="${VERSION#v}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_DIR}"

OUTPUT_DIR="${REPO_DIR}/target/release-assets"
mkdir -p "${OUTPUT_DIR}"

ARCH="amd64"
DEB_NAME="phairdlna_${VERSION_NUM}_${ARCH}"
STAGE_DIR="${REPO_DIR}/target/deb_staging/${DEB_NAME}"

echo "=== Packaging Linux Debian Package: ${DEB_NAME}.deb ==="
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/DEBIAN"
mkdir -p "${STAGE_DIR}/usr/bin"
mkdir -p "${STAGE_DIR}/lib/systemd/system"
mkdir -p "${STAGE_DIR}/usr/share/doc/phairdlna"

# 1. DEBIAN/control
cat <<EOF > "${STAGE_DIR}/DEBIAN/control"
Package: phairdlna
Version: ${VERSION_NUM}
Section: sound
Priority: optional
Architecture: ${ARCH}
Maintainer: PhairDLNA Contributors <https://github.com/mazer666/PhairDLNA>
Description: Honest, lightweight UPnP AV / DLNA media server
 PhairDLNA is an AGPL-3.0-or-later media server that serves audio, video,
 and pictures to smart televisions and UPnP AV control points without telemetry,
 cloud accounts, or unnecessary internet connections.
EOF

# 2. DEBIAN/postinst & prerm
cat <<'EOF' > "${STAGE_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    if command -v systemctl >/dev/null 2>&1; then
        systemctl daemon-reload || true
    fi
fi
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/postinst"

cat <<'EOF' > "${STAGE_DIR}/DEBIAN/prerm"
#!/bin/sh
set -e
if [ "$1" = "remove" ]; then
    if command -v systemctl >/dev/null 2>&1; then
        systemctl stop phairdlna.service || true
        systemctl disable phairdlna.service || true
    fi
fi
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/prerm"

# 3. Systemd service unit
cat <<EOF > "${STAGE_DIR}/lib/systemd/system/phairdlna.service"
[Unit]
Description=PhairDLNA media server
Documentation=https://github.com/mazer666/PhairDLNA
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
NotifyAccess=main
ExecStart=/usr/bin/phairdlna serve
Restart=on-failure
RestartSec=5
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
EOF

# 4. Binary & Documentation
if [[ -f "${REPO_DIR}/target/release/phairdlna-cli" ]]; then
    cp "${REPO_DIR}/target/release/phairdlna-cli" "${STAGE_DIR}/usr/bin/phairdlna"
elif [[ -f "${REPO_DIR}/target/x86_64-apple-darwin/release/phairdlna-cli" ]]; then
    cp "${REPO_DIR}/target/x86_64-apple-darwin/release/phairdlna-cli" "${STAGE_DIR}/usr/bin/phairdlna"
else
    # Build if needed
    PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --bin phairdlna-cli
    cp "${REPO_DIR}/target/release/phairdlna-cli" "${STAGE_DIR}/usr/bin/phairdlna"
fi
chmod 755 "${STAGE_DIR}/usr/bin/phairdlna"

cp "${REPO_DIR}/LICENSE" "${STAGE_DIR}/usr/share/doc/phairdlna/copyright"
cp "${REPO_DIR}/README.md" "${STAGE_DIR}/usr/share/doc/phairdlna/"

# 5. Build .deb package using dpkg-deb or fallback to ar/tar
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${STAGE_DIR}" "${OUTPUT_DIR}/${DEB_NAME}.deb"
else
    echo "dpkg-deb not present; building Debian archive layout with ar/tar..."
    cd "${STAGE_DIR}"
    echo "2.0" > "${STAGE_DIR}/debian-binary"
    tar -czf "${STAGE_DIR}/control.tar.gz" -C "${STAGE_DIR}/DEBIAN" .
    tar -czf "${STAGE_DIR}/data.tar.gz" --exclude="./DEBIAN" --exclude="./control.tar.gz" --exclude="./data.tar.gz" --exclude="./debian-binary" -C "${STAGE_DIR}" .
    ar -rc "${OUTPUT_DIR}/${DEB_NAME}.deb" "${STAGE_DIR}/debian-binary" "${STAGE_DIR}/control.tar.gz" "${STAGE_DIR}/data.tar.gz"
fi

rm -rf "${REPO_DIR}/target/deb_staging"
echo "SUCCESS: Debian package created at ${OUTPUT_DIR}/${DEB_NAME}.deb"
