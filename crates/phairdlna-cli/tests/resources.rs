//! The resources of the daemon.
//!
//! What: the tests of the scheduler of the daemon — the order in which the
//!       work of the classes runs, the report of the resources and the place
//!       of a transfer.
//! Why: the daemon has to answer a client before it reads the library
//!       (`RES-001`), and the numbers of the resources must not leak the files
//!       of the user (`RES-006`).
//! Example: the test holds the worker of the daemon with a background task
//!       and checks that the control plane is served first.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use phairdlna_cli::serve::log::{Level, Logger};
use phairdlna_cli::serve::resources::Resources;
use phairdlna_core::config::Config;
use phairdlna_core::security::Redactor;

/// How long a test waits for the daemon to do something.
const BUDGET: Duration = Duration::from_secs(5);

/// The resources of a daemon with the default configuration.
fn resources() -> Resources {
    let logger = Logger::with_redactor(Level::Info, "daemon", Arc::new(Redactor::new()));
    Resources::new(&Config::default(), &logger)
}

/// Waits until the answer is true, or until the budget is over.
fn wait_until(answer: impl Fn() -> bool) -> bool {
    let deadline = std::time::Instant::now() + BUDGET;
    while std::time::Instant::now() < deadline {
        if answer() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    answer()
}

/// The daemon answers a client before it reads the library.
#[test]
fn the_daemon_answers_a_client_before_it_reads_the_library() {
    let resources = resources();
    let released = Arc::new(AtomicBool::new(false));
    let answered = Arc::new(AtomicBool::new(false));

    // The background work takes the worker of the daemon and waits.
    let finish = Arc::clone(&released);
    resources
        .submit_background(move || {
            while !finish.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
        })
        .expect("the background task fits in the queue");

    // The control plane arrives behind it and is served first.
    let flag = Arc::clone(&answered);
    resources
        .submit(
            phairdlna_core::scheduler::Priority::ControlPlane,
            move || {
                flag.store(true, Ordering::SeqCst);
            },
        )
        .expect("the control task fits in the queue");

    assert!(
        wait_until(|| answered.load(Ordering::SeqCst)),
        "the control plane is served while the background work waits"
    );
    released.store(true, Ordering::SeqCst);
}

/// The report of the resources names no file and no address.
#[test]
fn the_report_of_the_resources_names_no_private_data() {
    let resources = resources();
    let report = resources.report();
    let summary = report.summary();

    assert!(
        summary.contains("streaming"),
        "the report names the classes"
    );
    assert!(
        !summary.contains('/'),
        "the report names no path: {summary}"
    );
    assert!(
        !summary.contains("192.168"),
        "the report names no address: {summary}"
    );
}

/// A transfer takes a place and gives it back when it ends.
#[test]
fn a_transfer_takes_a_place_and_gives_it_back() {
    let resources = resources();
    assert_eq!(resources.streams(), 0);

    let token = resources.stream_started();
    assert_eq!(resources.streams(), 1);

    drop(token);
    assert_eq!(resources.streams(), 0);
}
