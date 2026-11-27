#!/usr/bin/env bash
# End-to-end device and client interoperability test against a running PhairDLNA daemon.
# Tests SSDP discovery, HTTP endpoints, SOAP ContentDirectory, hierarchical browse,
# full HTTP media delivery, and HTTP Range requests.
#
# Requirements: INT-001..INT-003, INT-020, INT-023, INT-031, INT-080, INT-081, PC-06, PC-07, PC-08, PC-16, PC-17

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-50100}"
SSDP_PORT="${2:-50190}"

# Find free ports if defaults are busy
get_free_port() {
    python3 -c '
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind(("127.0.0.1", 0))
port = s.getsockname()[1]
s.close()
print(port)
'
}

get_free_udp_port() {
    python3 -c '
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("127.0.0.1", 0))
port = s.getsockname()[1]
s.close()
print(port)
'
}

ROOT_DIR="$(mktemp -d /tmp/phairdlna-device-test.XXXXXX)"
CONFIG_FILE="${ROOT_DIR}/config.toml"
MEDIA_DIR="${ROOT_DIR}/media"

mkdir -p "${MEDIA_DIR}"
echo "Test Video Content Bytes" > "${MEDIA_DIR}/sample.mp4"

cat << TOML > "${CONFIG_FILE}"
schema_version = 1

[server]
device_name = "PhairDLNA Test Device"
http_port = ${PORT}
ssdp_port = ${SSDP_PORT}
discovery_enabled = true
symlink_policy = "skip"

[library]
[[library.roots]]
name = "TestMedia"
path = "${MEDIA_DIR}"

[security]
allowed_clients = []
admin_enabled = false
TOML

echo "=== Starting PhairDLNA daemon (HTTP: ${PORT}, SSDP: ${SSDP_PORT}) ==="
cargo run --manifest-path "${REPO_DIR}/Cargo.toml" -p phairdlna-cli -- serve --config "${CONFIG_FILE}" --port "${PORT}" &
SERVER_PID=$!

cleanup() {
    echo "=== Stopping daemon (PID: ${SERVER_PID}) ==="
    kill -TERM "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
    rm -rf "${ROOT_DIR}"
}
trap cleanup EXIT

echo "Waiting for server to become responsive..."
for i in {1..30}; do
    if curl -s "http://127.0.0.1:${PORT}/description.xml" >/dev/null; then
        echo "Server is UP!"
        break
    fi
    sleep 0.5
done

# Step 1: Real SSDP M-SEARCH Discovery check
echo "=== 1. Checking SSDP M-SEARCH discovery (INT-001..INT-003, PC-01..PC-05) ==="
SSDP_PROBE=$(python3 - << PYEOF
import socket, sys

ssdp_port = ${SSDP_PORT}
client = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
client.settimeout(3.0)
msearch = (
    "M-SEARCH * HTTP/1.1\r\n"
    f"HOST: 239.255.255.250:{ssdp_port}\r\n"
    'MAN: "ssdp:discover"\r\n'
    "MX: 1\r\n"
    "ST: ssdp:all\r\n\r\n"
).encode("utf-8")

client.sendto(msearch, ("239.255.255.250", ssdp_port))
received = False
try:
    for _ in range(5):
        data, _ = client.recvfrom(2048)
        text = data.decode("utf-8", errors="replace")
        if "HTTP/1.1 200 OK" in text and ("ST: upnp:rootdevice" in text or "ST: urn:schemas-upnp-org:device:MediaServer:1" in text):
            received = True
            break
except socket.timeout:
    pass

if received:
    print("PASS")
    sys.exit(0)
else:
    print("FAIL")
    sys.exit(1)
PYEOF
)

if [ "${SSDP_PROBE}" = "PASS" ]; then
    echo "PASS: SSDP M-SEARCH discovery successfully answered"
else
    echo "FAIL: SSDP M-SEARCH discovery failed or timed out" >&2
    exit 1
fi

# Step 2: Device description check
echo "=== 2. Checking device description (INT-020, PC-06) ==="
DESC=$(curl -s "http://127.0.0.1:${PORT}/description.xml")
echo "${DESC}" | grep -q "urn:schemas-upnp-org:device:MediaServer:1"
echo "${DESC}" | grep -q "PhairDLNA Test Device"
echo "PASS: Device description valid"

# Step 3: ContentDirectory SCPD check
echo "=== 3. Checking ContentDirectory SCPD (INT-023, PC-07) ==="
SCPD=$(curl -s "http://127.0.0.1:${PORT}/ContentDirectory.xml")
echo "${SCPD}" | grep -q "urn:schemas-upnp-org:service-1-0"
echo "${SCPD}" | grep -q "Browse"
echo "PASS: ContentDirectory SCPD valid"

# Step 4: Hierarchical Browse to locate media resource URL
echo "=== 4. Testing Hierarchical Browse via SOAP (INT-031, PC-08) ==="
MEDIA_URL=$(python3 - << PYEOF
import urllib.request, xml.etree.ElementTree as ET, html, re, sys

port = ${PORT}

def browse(obj_id):
    soap = f"""<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
      <ObjectID>{obj_id}</ObjectID>
      <BrowseFlag>BrowseDirectChildren</BrowseFlag>
      <Filter>*</Filter>
      <StartingIndex>0</StartingIndex>
      <RequestedCount>20</RequestedCount>
      <SortCriteria></SortCriteria>
    </u:Browse>
  </s:Body>
</s:Envelope>"""
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/ctl/CD",
        data=soap.encode("utf-8"),
        headers={"Content-Type": "text/xml; charset=\"utf-8\"", "SOAPAction": "\"urn:schemas-upnp-org:service:ContentDirectory:1#Browse\""}
    )
    with urllib.request.urlopen(req) as resp:
        body = resp.read().decode("utf-8")
    m = re.search(r"<Result>(.*?)</Result>", body, re.DOTALL)
    if not m:
        return []
    didl = html.unescape(m.group(1))
    root = ET.fromstring(didl)
    entries = []
    for elem in root:
        elem_id = elem.attrib.get("id")
        tag = elem.tag.split("}")[-1]
        res = elem.find("{urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/}res")
        res_url = res.text if res is not None else None
        entries.append((tag, elem_id, res_url))
    return entries

# 1. Browse root "0" -> should contain container "1"
level0 = browse("0")
containers = [cid for tag, cid, _ in level0 if tag == "container"]
if not containers:
    sys.exit(1)

# 2. Browse container "1" -> should contain media root container
level1 = browse(containers[0])
root_containers = [cid for tag, cid, _ in level1 if tag == "container"]
if not root_containers:
    sys.exit(1)

# 3. Browse media root container -> should contain sample.mp4 item with resource URL
level2 = browse(root_containers[0])
items = [(cid, url) for tag, cid, url in level2 if tag == "item" and url]
if items and items[0][1]:
    print(items[0][1])
    sys.exit(0)

sys.exit(1)
PYEOF
)

if [ -z "${MEDIA_URL}" ]; then
    echo "FAIL: Failed to resolve media resource URL through CDS Browse hierarchy" >&2
    exit 1
fi
echo "PASS: Hierarchical Browse located media resource URL: ${MEDIA_URL}"

# Step 5: Full HTTP media delivery & Range request checks
echo "=== 5. Checking media HTTP delivery & byte ranges (INT-080, INT-081, PC-16, PC-17) ==="
FULL=$(curl -s "${MEDIA_URL}")
if [ "${FULL}" != "Test Video Content Bytes" ]; then
    echo "FAIL: Full HTTP media delivery returned unexpected content: '${FULL}'" >&2
    exit 1
fi
echo "PASS: Full media HTTP delivery verified"

RANGE=$(curl -s -H "Range: bytes=0-3" "${MEDIA_URL}")
if [ "${RANGE}" != "Test" ]; then
    echo "FAIL: HTTP Range request returned unexpected content: '${RANGE}'" >&2
    exit 1
fi
echo "PASS: Range HTTP delivery verified"

echo "=== ALL INTEROPERABILITY PROTOCOL CHECKS PASSED ==="
