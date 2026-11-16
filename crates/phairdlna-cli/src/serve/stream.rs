//! The body of a media answer.
//!
//! What: the bytes of a file, read in pieces and handed to the client.
//! Why: a film is larger than the memory of a server, and a client that stops
//!       reading must not cost the server the whole file (`INT-088`).
//! Example: `FileStream::open` opens a file and remembers how many bytes are
//!       still to come.

use std::io::SeekFrom;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Bytes;
use futures_core::Stream;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncSeekExt, ReadBuf};
use tokio::sync::OwnedSemaphorePermit;

/// The size of one piece of a transfer.
const CHUNK: usize = 64 * 1024;

/// The bytes of a file, read in pieces.
pub struct FileStream {
    file: File,
    remaining: u64,
    size: u64,
}

impl FileStream {
    /// Opens the file and positions it at the offset.
    ///
    /// The file is opened for reading only; the daemon never writes to a media
    /// file (`REL-001`).
    pub async fn open(path: PathBuf, offset: u64) -> Result<Self, std::io::Error> {
        let mut file = File::open(path).await?;
        let size = file.metadata().await?.len();
        if offset > 0 {
            file.seek(SeekFrom::Start(offset)).await?;
        }
        Ok(Self {
            file,
            remaining: 0,
            size,
        })
    }

    /// The size the file really has.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The number of bytes the stream carries.
    pub fn length(&self) -> u64 {
        self.remaining
    }

    /// Limits the stream to a number of bytes.
    ///
    /// The number may be larger than the file has; the stream then stops at
    /// the end of the file.
    pub fn limit(&mut self, length: u64) {
        self.remaining = length.min(self.size);
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, std::io::Error>;

    /// Reads the next piece of the file.
    ///
    /// The end of the file ends the stream; an error ends it with the error,
    /// so that the client learns that the transfer failed.
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.remaining == 0 {
            return Poll::Ready(None);
        }
        let wanted = this.remaining.min(CHUNK as u64) as usize;
        let mut buffer = vec![0u8; wanted];
        let read = {
            let mut read_buf = ReadBuf::new(&mut buffer);
            match Pin::new(&mut this.file).poll_read(context, &mut read_buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(error)) => return Poll::Ready(Some(Err(error))),
                Poll::Ready(Ok(())) => read_buf.filled().len(),
            }
        };
        if read == 0 {
            return Poll::Ready(None);
        }
        buffer.truncate(read);
        this.remaining -= read as u64;
        Poll::Ready(Some(Ok(Bytes::from(buffer))))
    }
}

/// A file stream that holds the place of a transfer.
///
/// The place is given back when the stream is dropped, and the stream is
/// dropped when the answer has been sent. A transfer that is cut off in the
/// middle therefore does not keep its place forever (`SEC-012`).
pub struct PermittedStream {
    /// The bytes of the file.
    inner: FileStream,
    /// The place of the transfer.
    _permit: OwnedSemaphorePermit,
}

impl PermittedStream {
    /// Builds a stream that holds the place of a transfer.
    pub fn new(inner: FileStream, permit: OwnedSemaphorePermit) -> Self {
        Self {
            inner,
            _permit: permit,
        }
    }
}

impl Stream for PermittedStream {
    type Item = Result<Bytes, std::io::Error>;

    /// Reads the next piece of the file.
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(context)
    }
}
