//! The transfers that are running.
//!
//! What: the list of the streams that are being played right now.
//! Why: the user of the machine wants to know that the server is busy, and by
//!       whom. The registry counts the transfers and names the clients; it
//!       never keeps the path of a file (`UX-012`, `SEC-094`).
//! Example: a client asks for a file, the transfer is registered, and the
//!          status of the server counts one stream more.

use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::Instant;

use futures_core::Stream;

/// The transfers that are running.
#[derive(Clone, Default)]
pub struct Transfers {
    /// The transfers that are running.
    inner: Arc<Mutex<Vec<Transfer>>>,
    /// The number that names the next transfer.
    counter: Arc<AtomicU64>,
}

/// One transfer that is running.
struct Transfer {
    /// The number that names the transfer.
    id: u64,
    /// The name of the client that plays the stream.
    client: String,
    /// What is being played.
    title: String,
    /// When the transfer started.
    started: Instant,
}

/// One transfer, as the status of the server shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningTransfer {
    /// The name of the client that plays the stream.
    pub client: String,
    /// What is being played.
    pub title: String,
    /// How long the transfer has been running, in seconds.
    pub running_seconds: u64,
}

impl Transfers {
    /// A registry without a transfer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a transfer and hands back the handle that ends it.
    ///
    /// The handle ends the transfer when it is dropped: the handle lives as
    /// long as the answer that carries the stream, and a stream that is cut
    /// off in the middle does not stay in the list forever.
    pub fn start(&self, client: &str, title: &str) -> TransferHandle {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let transfer = Transfer {
            id,
            client: client.to_owned(),
            title: title.to_owned(),
            started: Instant::now(),
        };
        self.write().push(transfer);
        TransferHandle {
            id,
            transfers: self.clone(),
        }
    }

    /// How many transfers are running.
    pub fn count(&self) -> usize {
        self.read().len()
    }

    /// The transfers that are running.
    pub fn list(&self) -> Vec<RunningTransfer> {
        self.read()
            .iter()
            .map(|transfer| RunningTransfer {
                client: transfer.client.clone(),
                title: transfer.title.clone(),
                running_seconds: transfer.started.elapsed().as_secs(),
            })
            .collect()
    }

    /// Ends a transfer.
    fn end(&self, id: u64) {
        self.write().retain(|transfer| transfer.id != id);
    }

    /// The transfers of the registry.
    fn read(&self) -> MutexGuard<'_, Vec<Transfer>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The transfers of the registry, for a change.
    fn write(&self) -> MutexGuard<'_, Vec<Transfer>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// The handle that ends a transfer when it is dropped.
pub struct TransferHandle {
    /// The number that names the transfer.
    id: u64,
    /// The registry the transfer belongs to.
    transfers: Transfers,
}

impl Drop for TransferHandle {
    /// Ends the transfer.
    fn drop(&mut self) {
        self.transfers.end(self.id);
    }
}

/// A stream that ends a transfer when it is dropped.
///
/// The stream lives as long as the answer that carries it: a transfer that is
/// cut off in the middle ends with the answer, and not before.
pub struct TrackedStream<S> {
    /// The bytes of the answer.
    inner: S,
    /// The transfer the answer carries.
    _transfer: TransferHandle,
}

impl<S> TrackedStream<S> {
    /// Builds a stream that ends a transfer when it is dropped.
    pub fn new(inner: S, transfer: TransferHandle) -> Self {
        Self {
            inner,
            _transfer: transfer,
        }
    }
}

impl<S: Stream + Unpin> Stream for TrackedStream<S> {
    type Item = S::Item;

    /// Reads the next piece of the answer.
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(context)
    }
}

#[cfg(test)]
mod tests {
    use super::Transfers;

    /// A registry without a transfer counts nothing.
    #[test]
    fn a_registry_without_a_transfer_counts_nothing() {
        let transfers = Transfers::new();
        assert_eq!(transfers.count(), 0);
    }

    /// A transfer that runs is counted, and it is counted with its client.
    #[test]
    fn a_transfer_is_counted_while_it_runs() {
        let transfers = Transfers::new();
        let handle = transfers.start("Living Room TV", "A film");

        assert_eq!(transfers.count(), 1);
        assert_eq!(transfers.list()[0].client, "Living Room TV");
        assert_eq!(transfers.list()[0].title, "A film");
        drop(handle);
    }

    /// A transfer that ended is not counted any more.
    #[test]
    fn a_transfer_that_ended_is_not_counted() {
        let transfers = Transfers::new();
        let handle = transfers.start("Living Room TV", "A film");

        drop(handle);
        assert_eq!(transfers.count(), 0);
    }

    /// Two transfers are counted separately.
    #[test]
    fn two_transfers_are_counted_separately() {
        let transfers = Transfers::new();
        let first = transfers.start("One", "A film");
        let second = transfers.start("Two", "Another film");

        assert_eq!(transfers.count(), 2);
        drop(first);
        assert_eq!(transfers.count(), 1);
        drop(second);
        assert_eq!(transfers.count(), 0);
    }
}
