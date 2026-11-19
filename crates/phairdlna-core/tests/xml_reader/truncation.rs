use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use phairdlna_core::upnp::xml_reader::parse;
use phairdlna_core::upnp::UpnpError;

const CHILD_ENV: &str = "PHAIRDLNA_XML_TRUNCATED_CHILD";
const COMPLETE_MARKER: &str = "truncated-xml-child-complete";
const TEST_NAME: &str = "truncation::truncated_text_terminates_and_reader_recovers";
const WATCHDOG: Duration = Duration::from_secs(2);
const DOCUMENTS: &[&str] = &[
    "<root>",
    "<root>unfinished",
    "<?xml version='1.0'",
    "<!-- unfinished",
    "<root><!-- unfinished",
    "<root><![CDATA[unfinished",
    "<root attr='unfinished",
    "<root attr='closed'",
    "<root><child",
    "<root></child",
    "<root>text</root",
];

/// Truncated XML fails under a watchdog, and the parser accepts valid input afterward.
#[test]
fn truncated_text_terminates_and_reader_recovers() {
    if let Some(nonce) = isolated_child_nonce() {
        assert_truncated_input_and_recovery();
        println!("{COMPLETE_MARKER}:{nonce}");
    } else {
        assert_isolated_parser_child();
    }
}

fn assert_truncated_input_and_recovery() {
    for document in DOCUMENTS {
        assert!(
            matches!(parse(document), Err(UpnpError::MalformedRequest { .. })),
            "truncated XML was not reported as malformed: {document:?}"
        );
    }
    assert_eq!(
        parse("<root>recovered</root>").expect("valid XML").text,
        "recovered"
    );
}

fn assert_isolated_parser_child() {
    let nonce = format!(
        "{}:{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after UNIX epoch")
            .as_nanos()
    );
    let child = Command::new(std::env::current_exe().expect("test executable path"))
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(CHILD_ENV, &nonce)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn isolated parser regression");
    let (status, output) = wait_with_timeout(child, WATCHDOG);
    assert!(
        status.success(),
        "isolated parser regression failed: {status}"
    );
    assert!(
        output.contains(&format!("{COMPLETE_MARKER}:{nonce}")),
        "test harness exited without running child assertions: {output:?}"
    );
}

fn isolated_child_nonce() -> Option<String> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let exact_filter = arguments
        .windows(2)
        .any(|pair| pair[0] == "--exact" && pair[1] == TEST_NAME);
    if !exact_filter {
        return None;
    }
    let nonce = std::env::var(CHILD_ENV).ok()?;
    let (process_id, timestamp) = nonce.split_once(':')?;
    process_id.parse::<u32>().ok()?;
    timestamp.parse::<u128>().ok()?;
    Some(nonce)
}

fn wait_with_timeout(mut child: Child, timeout: Duration) -> (ExitStatus, String) {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status, read_completion_marker(&mut child)),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("truncated XML parser exceeded {timeout:?}");
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("could not wait for parser child: {error}");
            }
        }
    }
}

fn read_completion_marker(child: &mut Child) -> String {
    let mut output = String::new();
    child
        .stdout
        .take()
        .expect("child stdout is piped")
        .read_to_string(&mut output)
        .expect("read child completion marker");
    output
}
