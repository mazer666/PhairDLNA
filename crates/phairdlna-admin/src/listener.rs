//! The bounded socket listener for the local administration API.
//!
//! What: caps accepted connections and enforces read/write silence deadlines.
//! Why: router timeouts start after HTTP headers, so idle sockets need an I/O boundary.
//! Example: each accepted stream is wrapped before Hyper can read its first byte.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::serve::Listener;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, Sleep};

const ACCEPT_PAUSE: Duration = Duration::from_millis(25);

/// A listener that shares a connection budget and attaches silence deadlines.
pub(crate) struct AdminListener {
    listener: TcpListener,
    budget: std::sync::Arc<Semaphore>,
    read_timeout: Duration,
    write_timeout: Duration,
}

impl AdminListener {
    /// Wraps a socket using the shared API connection budget and configured timeouts.
    pub(crate) fn new(
        listener: TcpListener,
        budget: std::sync::Arc<Semaphore>,
        read_timeout: Duration,
        write_timeout: Duration,
    ) -> Self {
        Self {
            listener,
            budget,
            read_timeout,
            write_timeout,
        }
    }
}

impl Listener for AdminListener {
    type Io = DeadlineIo<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (stream, address) = match self.listener.accept().await {
                Ok(connection) => connection,
                Err(_) => {
                    tokio::time::sleep(ACCEPT_PAUSE).await;
                    continue;
                }
            };
            let permit = match self.budget.clone().try_acquire_owned() {
                Ok(permit) => permit,
                Err(_) => continue,
            };
            return (
                DeadlineIo::new(stream, self.read_timeout, self.write_timeout, permit),
                address,
            );
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.listener.local_addr()
    }
}

/// I/O wrapper that releases its connection permit when the connection closes.
pub(crate) struct DeadlineIo<T> {
    inner: T,
    read_deadline: Pin<Box<Sleep>>,
    write_deadline: Pin<Box<Sleep>>,
    read_timeout: Duration,
    write_timeout: Duration,
    _permit: OwnedSemaphorePermit,
}

impl<T> DeadlineIo<T> {
    fn new(
        inner: T,
        read_timeout: Duration,
        write_timeout: Duration,
        permit: OwnedSemaphorePermit,
    ) -> Self {
        Self {
            inner,
            read_deadline: Box::pin(tokio::time::sleep(read_timeout)),
            write_deadline: Box::pin(tokio::time::sleep(write_timeout)),
            read_timeout,
            write_timeout,
            _permit: permit,
        }
    }

    fn reset_read_deadline(&mut self) {
        self.read_deadline
            .as_mut()
            .reset(Instant::now() + self.read_timeout);
    }

    fn reset_write_deadline(&mut self) {
        self.write_deadline
            .as_mut()
            .reset(Instant::now() + self.write_timeout);
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for DeadlineIo<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.read_deadline.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(timed_out("the request read timed out")));
        }
        let before = buffer.filled().len();
        match Pin::new(&mut this.inner).poll_read(context, buffer) {
            Poll::Ready(Ok(())) => {
                if buffer.filled().len() > before {
                    this.reset_read_deadline();
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for DeadlineIo<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if this.write_deadline.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(timed_out("the response write timed out")));
        }
        match Pin::new(&mut this.inner).poll_write(context, buffer) {
            Poll::Ready(Ok(written)) => {
                if written > 0 {
                    this.reset_write_deadline();
                }
                Poll::Ready(Ok(written))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        poll_write_operation(self, context, |inner, context| {
            Pin::new(inner).poll_flush(context)
        })
    }

    fn poll_shutdown(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        poll_write_operation(self, context, |inner, context| {
            Pin::new(inner).poll_shutdown(context)
        })
    }
}

fn poll_write_operation<T: AsyncWrite + Unpin>(
    mut this: Pin<&mut DeadlineIo<T>>,
    context: &mut Context<'_>,
    mut operation: impl FnMut(&mut T, &mut Context<'_>) -> Poll<io::Result<()>>,
) -> Poll<io::Result<()>> {
    let state = this.as_mut().get_mut();
    if state.write_deadline.as_mut().poll(context).is_ready() {
        return Poll::Ready(Err(timed_out("the response write timed out")));
    }
    operation(&mut state.inner, context)
}

fn timed_out(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, message)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    const SHORT: Duration = Duration::from_millis(20);

    #[tokio::test]
    async fn idle_reads_expire() {
        let (stream, _peer) = tokio::io::duplex(8);
        let permit = Arc::new(Semaphore::new(1))
            .try_acquire_owned()
            .expect("permit is available");
        let mut stream = DeadlineIo::new(stream, SHORT, SHORT, permit);
        let error = stream.read_u8().await.expect_err("read must time out");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test]
    async fn blocked_writes_expire() {
        let (stream, _peer) = tokio::io::duplex(8);
        let permit = Arc::new(Semaphore::new(1))
            .try_acquire_owned()
            .expect("permit is available");
        let mut stream = DeadlineIo::new(stream, SHORT, SHORT, permit);
        let error = stream
            .write_all(&[b'x'; 64])
            .await
            .expect_err("blocked write must time out");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test]
    async fn wrapped_connection_holds_and_releases_its_permit() {
        let budget = Arc::new(Semaphore::new(1));
        let permit = budget
            .clone()
            .try_acquire_owned()
            .expect("connection budget is available");
        let (stream, _peer) = tokio::io::duplex(8);
        let connection = DeadlineIo::new(stream, SHORT, SHORT, permit);

        assert!(budget.clone().try_acquire_owned().is_err());
        drop(connection);
        assert!(budget.try_acquire().is_ok());
    }
}
