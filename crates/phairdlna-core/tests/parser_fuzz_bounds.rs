//! Parser robustness, boundary limits, and malformed input test suite.
//!
//! What: fuzz-like malformed inputs, extreme bounds, and property validation (`M4.3`, `SEC-060`..`SEC-065`, `SEC-068`).
//! Why: guarantees that invalid or pathological inputs reject gracefully without panicking or exhausting resources.

mod support;

use phairdlna_core::security::containment::resolve_reference;
use phairdlna_core::security::Limits;
use phairdlna_core::ssdp::replay_datagram;
use phairdlna_core::ssdp::{DeviceIdentity, ServerIdentity};
use phairdlna_core::upnp::{Dispatcher, PAGE_LIMIT};
use std::path::PathBuf;
use support::catalogue::TestCatalogue;

fn test_device() -> DeviceIdentity {
    DeviceIdentity::new("12345678-1234-1234-1234-123456789abc", "Living Room TV")
}

fn test_server() -> ServerIdentity {
    ServerIdentity {
        friendly_name: "Living Room TV".to_owned(),
        product: "PhairDLNA".to_owned(),
        version: "0.1.0".to_owned(),
    }
}

/// Simple pseudorandom generator for deterministic reproducible testing.
struct Lcg(u64);
impl Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }
    fn next_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| (self.next_u32() & 0xFF) as u8).collect()
    }
}

#[test]
fn ssdp_fuzzed_datagrams_never_panic() {
    let mut rng = Lcg(0xdead_beef_1234_5678);
    let device = test_device();
    let server = test_server();

    for _ in 0..500 {
        let len = (rng.next_u32() % 1024) as usize;
        let bytes = rng.next_bytes(len);
        let raw = String::from_utf8_lossy(&bytes);

        // Replaying garbage datagrams must return Err(Ignored) or Ok, but never panic.
        let _ = replay_datagram(&raw, &device, &server, "192.168.1.1", 8200);
    }
}

#[test]
fn playlist_network_and_traversal_paths_contained() {
    let dangerous_entries = [
        "http://attacker.com/malicious.mkv",
        "https://evil.org/track.mp3",
        "ftp://server/movie.avi",
        "file:///etc/passwd",
        "smb://192.168.1.1/share",
        "../../../../../../../../etc/shadow",
        "..\\..\\..\\..\\windows\\system32\\cmd.exe",
        "/etc/passwd",
    ];

    let root = PathBuf::from("/media/library");
    let roots = vec![root];
    let playlist_folder = PathBuf::from("/media/library/playlists");
    for entry in dangerous_entries {
        let res = resolve_reference(entry, &playlist_folder, &roots);
        assert!(
            res.is_err(),
            "dangerous entry \"{entry}\" must be rejected by containment"
        );
    }
}

#[test]
fn soap_oversized_envelope_rejected_by_limits() {
    let limits = Limits::default();
    let oversized = "A".repeat(limits.body_bytes + 10);
    assert!(!limits.accepts_body(oversized.len()));

    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let response = dispatcher.dispatch(&oversized, None);

    assert!(response.status >= 400 || response.status == 500);
}

#[test]
fn browse_extreme_counts_bounded_to_page_limit() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");

    let extreme_counts = [u32::MAX, u32::MAX - 1, 2_147_483_647, 501, 100_000];
    for count in extreme_counts {
        let body = support::browse_request("1", "BrowseDirectChildren", 0, count);
        let response = dispatcher.dispatch(&body, None);
        assert_eq!(response.status, 200);
        let returned = support::element_text(&response.body, "NumberReturned")
            .and_then(|t| t.parse::<usize>().ok())
            .unwrap_or(0);
        assert!(
            returned <= PAGE_LIMIT,
            "returned count {returned} exceeds limit {PAGE_LIMIT}"
        );
    }
}
