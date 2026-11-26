#!/usr/bin/env bash
# Generates release checksums and software bill of materials (SBOM) (SUP-005, PLT-016).
# Ensures release policy compliance: accompanied by SHA-256 checksums and dependency bill of materials.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_DIR}"

DIST_DIR="${REPO_DIR}/target/dist"
mkdir -p "${DIST_DIR}"

echo "=== Generating Release Artefacts & SBOM (SUP-005) ==="

# 1. Build release binaries
PATH="${HOME}/.cargo/bin:${PATH}" cargo build --release --bin phairdlna-cli

CLI_BIN="${REPO_DIR}/target/release/phairdlna-cli"
if [[ ! -f "${CLI_BIN}" ]]; then
    echo "ERROR: Binary ${CLI_BIN} not found." >&2
    exit 1
fi

TARBALL="${DIST_DIR}/phairdlna-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz"
tar -czf "${TARBALL}" -C "${REPO_DIR}/target/release" phairdlna-cli

# 2. Generate SHA-256 Checksum
echo "Computing SHA-256 checksum..."
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${TARBALL}" > "${TARBALL}.sha256"
else
    shasum -a 256 "${TARBALL}" > "${TARBALL}.sha256"
fi

# 3. Generate Software Bill of Materials (SBOM) in JSON and plain text format
echo "Generating Software Bill of Materials (SBOM)..."
SBOM_FILE="${DIST_DIR}/phairdlna-sbom.json"
python3 -c "
import json
import subprocess

out = subprocess.check_output(['cargo', 'metadata', '--format-version', '1']).decode('utf-8')
metadata = json.loads(out)

packages = []
for pkg in metadata.get('packages', []):
    packages.append({
        'name': pkg.get('name'),
        'version': pkg.get('version'),
        'license': pkg.get('license'),
        'source': pkg.get('source'),
        'repository': pkg.get('repository')
    })

sbom = {
    'bomFormat': 'PhairDLNA-SBOM',
    'specVersion': '1.0',
    'product': 'PhairDLNA',
    'total_components': len(packages),
    'components': sorted(packages, key=lambda x: x['name'])
}

with open('${SBOM_FILE}', 'w') as f:
    json.dump(sbom, f, indent=2)

print(f'SBOM generated with {len(packages)} components at ${SBOM_FILE}')
"

echo "SUCCESS: Release artefacts generated in ${DIST_DIR}:"
ls -la "${DIST_DIR}"
