//! The bounds of the connections.
//!
//! What: the number of connections the server opens at the same time, and the
//!       deadline every read and every write carries.
//! Why: a server that accepts every connection and waits for ever on a
//!       client that never speaks is a server that one host on the network can
//!       take down (`SEC-012`, `SEC-016`). The bounds are here, not in the
//!       core, because they are properties of the socket (`ADR-0001`).
//! Example: a client that opens a connection and says nothing is cut off when
//!       the read deadline passes; a connection beyond the bound is closed at
//!       once, and the open ones go on.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Instant, Sleep};

/// The pause between two attempts to accept a connection.
///
/// The pause keeps a socket that is broken from spinning the daemon.
const ACCEPT_PAUSE: Duration = Duration::from_millis(50);

/// The number of connections that may be open at the same time.
///
/// The count is the number of connections the server is holding, not the
/// number of clients it has seen: a connection that ends gives its place
/// back.
#[derive(Debug, Clone)]
pub struct ConnectionBudget {
    /// The number of open connections.
    open: Arc<AtomicUsize>,
    /// The number of connections that may be open.
    limit: usize,
}

impl ConnectionBudget {
    /// A budget that allows the given number of connections.
    pub fn new(limit: usize) -> Self {
        Self {
            open: Arc::new(AtomicUsize::new(0)),
            limit,
        }
    }

    /// Whether a new connection may be opened.
    ///
    /// The check and the count happen together, so that two connections that
    /// arrive at the same moment cannot both slip past the bound.
    pub fn admits(&self) -> bool {
        let mut current = self.open.load(Ordering::SeqCst);
        loop {
            if current >= self.limit {
                return false;
            }
            match self.open.compare_exchange_weak(
                current,
                current + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    /// The number of connections that are open.
    pub fn open(&self) -> usize {
        self.open.load(Ordering::SeqCst)
    }

    /// Gives the place of a connection back.
    pub fn release(&self) {
        self.open.fetch_sub(1, Ordering::SeqCst);
    }
}

/// The listener of the server: it accepts connections, and it keeps them
/// inside the bounds.
pub struct CappedListener {
    /// The socket that listens.
    listener: TcpListener,
    /// The number of connections that may be open.
    budget: ConnectionBudget,
    /// How long a connection may stay idle.
    read_timeout: Duration,
    /// How long the server may take to write an answer.
    write_timeout: Duration,
}

impl CappedListener {
    /// Builds a listener that keeps the connections inside the bounds.
    pub fn new(
        listener: TcpListener,
        limit: usize,
        read_timeout: Duration,
        write_timeout: Duration,
    ) -> Self {
        Self {
            listener,
            budget: ConnectionBudget::new(limit.max(1)),
            read_timeout,
            write_timeout,
        }
    }

    /// The number of connections that are open.
    pub fn open_connections(&self) -> usize {
        self.budget.open()
    }
}

impl axum::serve::Listener for CappedListener {
    /// The connection the server reads and writes.
    type Io = DeadlineIo;
    /// The address of the peer.
    type Addr = SocketAddr;

    /// Accepts a connection, if the bound allows it.
    ///
    /// A connection beyond the bound is closed at once: the client sees a
    /// connection that ends before a word was written, and the connections
    /// that are open go on (`SEC-012`).
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (stream, address) = match self.listener.accept().await {
                Ok(pair) => pair,
                Err(_) => {
                    tokio::time::sleep(ACCEPT_PAUSE).await;
                    continue;
                }
            };
            if !self.budget.admits() {
                continue;
            }
            let connection = DeadlineIo::new(
                stream,
                self.read_timeout,
                self.write_timeout,
                self.budget.clone(),
            );
            return (connection, address);
        }
    }

    /// The address the server listens on.
    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.listener.local_addr()
    }
}

/// A connection with a deadline on every read and every write.
///
/// The deadline is a deadline of *silence*: it is set again after every piece
/// that was read or written. A client that speaks slowly is served; a client
/// that stops speaking is cut off (`SEC-016`).
pub struct DeadlineIo {
    /// The socket.
    stream: TcpStream,
    /// When the next read must have happened.
    read_deadline: Pin<Box<Sleep>>,
    /// When the next write must have happened.
    write_deadline: Pin<Box<Sleep>>,
    /// How long a connection may stay idle.
    read_timeout: Duration,
    /// How long the server may take to write an answer.
    write_timeout: Duration,
    /// The place of this connection in the budget.
    budget: ConnectionBudget,
}

impl DeadlineIo {
    /// Builds a connection that carries the deadlines.
    fn new(
        stream: TcpStream,
        read_timeout: Duration,
        write_timeout: Duration,
        budget: ConnectionBudget,
    ) -> Self {
        Self {
            stream,
            read_deadline: Box::pin(tokio::time::sleep(read_timeout)),
            write_deadline: Box::pin(tokio::time::sleep(write_timeout)),
            read_timeout,
            write_timeout,
            budget,
        }
    }
}

impl Drop for DeadlineIo {
    /// Gives the place of the connection back.
    fn drop(&mut self) {
        self.budget.release();
    }
}

impl AsyncRead for DeadlineIo {
    /// Reads the next piece of the connection.
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);
        match stream.poll_read(context, buffer) {
            Poll::Ready(result) => {
                this.read_deadline
                    .as_mut()
                    .reset(Instant::now() + this.read_timeout);
                Poll::Ready(result)
            }
            Poll::Pending => match this.read_deadline.as_mut().poll(context) {
                Poll::Ready(_) => Poll::Ready(Err(timed_out("the connection timed out"))),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

impl AsyncWrite for DeadlineIo {
    /// Writes the next piece of the answer.
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let stream = Pin::new(&mut this.stream);
        match stream.poll_write(context, buffer) {
            Poll::Ready(result) => {
                this.write_deadline
                    .as_mut()
                    .reset(Instant::now() + this.write_timeout);
                Poll::Ready(result)
            }
            Poll::Pending => match this.write_deadline.as_mut().poll(context) {
                Poll::Ready(_) => Poll::Ready(Err(timed_out("the answer timed out"))),
                Poll::Pending => Poll::Pending,
            },
        }
    }

    /// Flushes the answer.
    fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.stream).poll_flush(context)
    }

    /// Closes the answer.
    fn poll_shutdown(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.stream).poll_shutdown(context)
    }
}

/// The error of a connection that ran out of time.
fn timed_out(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, message)
}

#[cfg(test)]
mod tests {
    use super::ConnectionBudget;

    /// The budget lets the connections in until the bound is reached.
    #[test]
    fn the_budget_counts_the_connections() {
        let budget = ConnectionBudget::new(2);

        assert!(budget.admits());
        assert!(budget.admits());
        assert!(!budget.admits(), "the third connection is refused");
        assert_eq!(budget.open(), 2);
    }

    /// A place that is given back lets the next connection in.
    #[test]
    fn a_place_is_given_back() {
        let budget = ConnectionBudget::new(1);

        assert!(budget.admits());
        assert!(!budget.admits());
        budget.release();
        assert!(budget.admits(), "the place of the connection is free again");
    }
}
