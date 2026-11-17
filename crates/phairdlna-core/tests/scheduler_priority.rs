//! The arbitration of the scheduler.
//!
//! What: the tests of the priority order of the classes — a stream and a
//!       control request come before the work of the library — and of the
//!       isolation of a slow client.
//! Why: `RES-001` asks for one scheduler with explicit priority classes, and
//!      `RES-004` asks that a slow client is not mistaken for a busy server.
//!      Both are only true if the order of the work can be observed.
//! Example: the test submits a scan that blocks, then a scan and a stream,
//!      and reads the order in which the worker took them.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairdlna_core::scheduler::{
    ClassLimits, Priority, Scheduler, SchedulerLimits, SchedulerOptions,
};

/// How long a test waits for the scheduler to do something.
const BUDGET: Duration = Duration::from_secs(5);

/// A scheduler with one worker and the given bounds.
fn scheduler_with(limits: SchedulerLimits) -> Scheduler {
    Scheduler::new(SchedulerOptions {
        limits,
        ..SchedulerOptions::default()
    })
    .expect("the bounds of the test are usable")
}

/// The limits of a test: one worker, so that the order of the work is clear.
fn one_worker() -> SchedulerLimits {
    SchedulerLimits {
        workers: 1,
        streaming: ClassLimits::new(4, 8),
        control_plane: ClassLimits::new(4, 8),
        media: ClassLimits::new(4, 8),
        background: ClassLimits::new(4, 8),
    }
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

/// The order in which the tasks ended.
type Order = Arc<Mutex<Vec<String>>>;

/// Remembers that a task of a name ended.
fn note(order: &Order, name: &str) {
    order
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(name.to_owned());
}

/// The names in the order they were noted.
fn names(order: &Order) -> Vec<String> {
    order
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// A stream runs before a scan that was waiting for it.
#[test]
fn a_stream_runs_before_the_scan_that_waits_for_it() {
    let scheduler = scheduler_with(one_worker());
    let order: Order = Arc::new(Mutex::new(Vec::new()));
    let (started, is_started) = channel();
    let (release, is_released) = channel();

    // The first task takes the only worker and waits.
    let first_order = Arc::clone(&order);
    scheduler
        .submit(Priority::Background, move || {
            note(&first_order, "scan-1");
            let _ = started.send(());
            let _ = is_released.recv();
        })
        .expect("the first task fits in the queue");
    is_started.recv().expect("the first task runs");

    // The scan waits in the queue, and the stream arrives behind it.
    let scan_order = Arc::clone(&order);
    scheduler
        .submit(Priority::Background, move || note(&scan_order, "scan-2"))
        .expect("the scan fits in the queue");
    let stream_order = Arc::clone(&order);
    scheduler
        .submit(Priority::Streaming, move || note(&stream_order, "stream"))
        .expect("the stream fits in the queue");

    release.send(()).expect("the first task is released");
    let done = wait_until(|| names(&order).len() >= 3);

    assert!(done, "the three tasks end");
    assert_eq!(
        names(&order),
        vec![
            "scan-1".to_owned(),
            "stream".to_owned(),
            "scan-2".to_owned()
        ],
        "the stream is taken before the scan that waits behind it"
    );
}

/// A control request runs before a scan that was waiting for it.
#[test]
fn a_control_request_runs_before_the_scan_that_waits_for_it() {
    let scheduler = scheduler_with(one_worker());
    let order: Order = Arc::new(Mutex::new(Vec::new()));
    let (started, is_started) = channel();
    let (release, is_released) = channel();

    let first_order = Arc::clone(&order);
    scheduler
        .submit(Priority::Background, move || {
            note(&first_order, "scan-1");
            let _ = started.send(());
            let _ = is_released.recv();
        })
        .expect("the first task fits in the queue");
    is_started.recv().expect("the first task runs");

    let scan_order = Arc::clone(&order);
    scheduler
        .submit(Priority::Background, move || note(&scan_order, "scan-2"))
        .expect("the scan fits in the queue");
    let control_order = Arc::clone(&order);
    scheduler
        .submit(Priority::ControlPlane, move || {
            note(&control_order, "control")
        })
        .expect("the control request fits in the queue");

    release.send(()).expect("the first task is released");
    let done = wait_until(|| names(&order).len() >= 3);

    assert!(done, "the three tasks end");
    assert_eq!(
        names(&order),
        vec![
            "scan-1".to_owned(),
            "control".to_owned(),
            "scan-2".to_owned()
        ],
        "the control request is taken before the scan that waits behind it"
    );
}

/// A slow client slows down its own stream and nobody else.
#[test]
fn a_slow_client_does_not_hold_up_another_stream() {
    let limits = SchedulerLimits {
        workers: 2,
        streaming: ClassLimits::new(4, 8),
        ..SchedulerLimits::default()
    };
    let scheduler = scheduler_with(limits);
    let (started, is_started) = channel();
    let (release, is_released) = channel();
    let (quick, is_quick) = channel();

    // The slow client reads its stream and takes its time.
    scheduler
        .submit(Priority::Streaming, move || {
            let _ = started.send(());
            let _ = is_released.recv();
        })
        .expect("the slow stream fits in the queue");
    is_started.recv().expect("the slow stream runs");

    // The second client is served while the first one is still waiting.
    scheduler
        .submit(Priority::Streaming, move || {
            let _ = quick.send(());
        })
        .expect("the second stream fits in the queue");

    let served = is_quick.recv_timeout(BUDGET);
    release.send(()).expect("the slow stream is released");

    assert!(
        served.is_ok(),
        "the second client is served while the first one is slow"
    );
}

/// A class runs no more tasks at the same time than its bound says.
#[test]
fn a_class_runs_no_more_tasks_than_its_bound() {
    let limits = SchedulerLimits {
        workers: 4,
        background: ClassLimits::new(1, 8),
        ..SchedulerLimits::default()
    };
    let scheduler = scheduler_with(limits);
    let running = Arc::new(AtomicUsize::new(0));
    let highest = Arc::new(AtomicUsize::new(0));
    let released = Arc::new(AtomicBool::new(false));

    for _ in 0..3 {
        let counter = Arc::clone(&running);
        let highest = Arc::clone(&highest);
        let finish = Arc::clone(&released);
        scheduler
            .submit(Priority::Background, move || {
                let now = counter.fetch_add(1, Ordering::SeqCst) + 1;
                highest.fetch_max(now, Ordering::SeqCst);
                while !finish.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(2));
                }
                counter.fetch_sub(1, Ordering::SeqCst);
            })
            .expect("the task fits in the queue");
    }

    // The first task runs and waits; the other two stay in the queue.
    assert!(wait_until(|| running.load(Ordering::SeqCst) == 1));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        highest.load(Ordering::SeqCst),
        1,
        "the background class runs one task at a time"
    );
    released.store(true, Ordering::SeqCst);
}
