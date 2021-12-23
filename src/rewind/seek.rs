use super::Rewind;
use crate::{ParseError, Result};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_io::{AsyncRead, AsyncSeek};
use pin_project_lite::pin_project;

pin_project! {
    /// A faster implementation on `Rewind` using `AsyncSeek`.
    ///
    /// Since `SeekInput` is faster than `BufferInput`, use this one as far as reader implements
    /// `AsyncSeek`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SeekInput<R> {
        #[pin]
        inner: R,
    }
}

impl<R> From<R> for SeekInput<R> {
    fn from(reader: R) -> Self {
        Self { inner: reader }
    }
}

impl<R> SeekInput<R> {
    /// Create a new object, same as `SeekInput::from(reader)`.
    pub fn new(reader: R) -> Self {
        Self::from(reader)
    }

    /// Consumes this object, returning inner reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: AsyncRead> AsyncRead for SeekInput<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<R: AsyncSeek> Rewind for SeekInput<R> {
    type Checkpoint = u64;
    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        self.project()
            .inner
            .poll_seek(cx, io::SeekFrom::Current(0))
            .map_ok(|pos| pos as usize)
            .map_err(|inner| ParseError::ReadError { inner })
    }

    fn poll_set_checkpoint(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        self.project()
            .inner
            .poll_seek(cx, io::SeekFrom::Current(0))
            .map_err(|inner| ParseError::ReadError { inner })
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        self.project()
            .inner
            .poll_seek(cx, io::SeekFrom::Start(checkpoint))
            .map_ok(|_| ())
            .map_err(|inner| ParseError::ReadError { inner })
    }
}
