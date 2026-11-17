//! The bounds of the scheduler.
//!
//! What: the tests of the refusal of the server when a bound is reached.
//! Why: `RES-002` asks for a documented bound and a defined overload
//!      behaviour, and `RES-007` asks for a meaningful explanation instead of
//!      a silent failure. A bound that cannot be reached proves nothing, so
//!      the tests reach it (`AC-10`).
//! Example: the test fills the queue of the background class and checks that
//!      the next task is refused with an error that names the bound.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use phairdlna_core::scheduler::{
    ClassLimits, Overload, Priority, Scheduler, SchedulerLimits, SchedulerOptions,
};

/// How long a test waits for the scheduler to do something.
const BUDGET: Duration = Duration::from_secs(5);

/// The scheduler of the tests: one worker and a queue of one task.
fn scheduler_with(queue: usize) -> Scheduler {
    Scheduler::new(SchedulerOptions {
        limits: SchedulerLimits {
            workers: 1,
            background: ClassLimits::new(1, queue),
            ..SchedulerLimits::default()
        },
        ..SchedulerOptions::default()
    })
    .expect("the bounds of the test are usable")
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

/// A full queue refuses the next task and says which bound was reached.
#[test]
fn a_full_queue_refuses_the_next_task_with_a_clear_error() {
    let scheduler = scheduler_with(1);
    let running = Arc::new(AtomicBool::new(false));
    let released = Arc::new(AtomicBool::new(false));

    // The first task takes the worker and waits there.
    let is_running = Arc::clone(&running);
    let finish = Arc::clone(&released);
    scheduler
        .submit(Priority::Background, move || {
            is_running.store(true, Ordering::SeqCst);
            while !finish.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
        })
        .expect("the first task fits in the queue");
    assert!(wait_until(|| running.load(Ordering::SeqCst)));

    // The second task fills the queue of the class.
    scheduler
        .submit(Priority::Background, || {})
        .expect("the second task fits in the queue");

    // The third task does not fit and is refused.
    let refused = scheduler.submit(Priority::Background, || {});
    assert_eq!(
        refused,
        Err(Overload::QueueFull {
            class: "background",
            capacity: 1
        }),
        "the task that does not fit is refused with the bound that was reached"
    );

    let message = refused.expect_err("the task was refused").to_string();
    assert!(
        message.contains("background"),
        "the message names the class"
    );
    assert!(message.contains('1'), "the message names the bound");
    released.store(true, Ordering::SeqCst);
}

/// The refusal is counted, so that the report can show it.
#[test]
fn a_refused_task_is_counted() {
    let scheduler = scheduler_with(1);
    let released = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(false));
    let is_running = Arc::clone(&running);
    let finish = Arc::clone(&released);
    scheduler
        .submit(Priority::Background, move || {
            is_running.store(true, Ordering::SeqCst);
            while !finish.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
        })
        .expect("the first task fits in the queue");
    assert!(wait_until(|| running.load(Ordering::SeqCst)));
    scheduler
        .submit(Priority::Background, || {})
        .expect("the second task fits in the queue");
    let _ = scheduler.submit(Priority::Background, || {});

    assert_eq!(scheduler.stats().background.rejected, 1);
    released.store(true, Ordering::SeqCst);
}

/// A scheduler that is stopping refuses every task.
#[test]
fn a_stopping_scheduler_refuses_every_task() {
    let scheduler = scheduler_with(4);
    scheduler.shutdown();

    for class in Priority::ALL {
        let refused = scheduler.submit(class, || {});
        assert_eq!(
            refused,
            Err(Overload::Closed),
            "the {} class is refused while the scheduler stops",
            class.as_str()
        );
    }
}

/// The number of tasks that run at the same time never passes the bound.
#[test]
fn the_number_of_running_tasks_never_passes_the_bound() {
    let limits = SchedulerLimits {
        workers: 8,
        streaming: ClassLimits::new(2, 16),
        ..SchedulerLimits::default()
    };
    let scheduler = Scheduler::new(SchedulerOptions {
        limits,
        ..SchedulerOptions::default()
    })
    .expect("the bounds of the test are usable");
    let running = Arc::new(AtomicUsize::new(0));
    let highest = Arc::new(AtomicUsize::new(0));
    let released = Arc::new(AtomicBool::new(false));

    for _ in 0..8 {
        let counter = Arc::clone(&running);
        let peak = Arc::clone(&highest);
        let finish = Arc::clone(&released);
        scheduler
            .submit(Priority::Streaming, move || {
                let now = counter.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                while !finish.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(2));
                }
                counter.fetch_sub(1, Ordering::SeqCst);
            })
            .expect("the task fits in the queue");
    }

    std::thread::sleep(Duration::from_millis(100));
    assert!(
        highest.load(Ordering::SeqCst) <= 2,
        "the streaming class runs no more than two tasks at a time"
    );
    released.store(true, Ordering::SeqCst);
}

/// The report of the scheduler names the bounds it keeps.
#[test]
fn the_report_names_the_bounds_it_keeps() {
    let scheduler = scheduler_with(3);
    let report = scheduler.report();
    let background = report
        .class(Priority::Background)
        .expect("the background class is in the report");
    assert_eq!(background.concurrent_limit, 1);
    assert_eq!(background.queue_limit, 3);
    assert_eq!(report.workers, 1);
}
