mod inner;

use super::Rewind;
use crate::Result;
use inner::Buffer;

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::AsyncRead;
use pin_project_lite::pin_project;

pin_project! {
    /// A `Rewind` implementation using buffers.
    ///
    /// `BufferInput` requires only `AsyncRead` as base types, but slower than `SeekInput`. For
    /// types implementing `AsyncSeek`, use `SeekInput` instead.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BufferInput<R> {
        position: usize,
        buffer: Buffer,
        #[pin]
        inner: R,
    }
}

impl<R> From<R> for BufferInput<R> {
    fn from(reader: R) -> Self {
        Self {
            position: 0,
            buffer: Buffer::default(),
            inner: reader,
        }
    }
}

impl<R> BufferInput<R> {
    /// Create a new object, same as `BufferInput::from(reader)`.
    pub fn new(reader: R) -> Self {
        Self::from(reader)
    }

    /// Consumes this object, returning inner reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: AsyncRead> AsyncRead for BufferInput<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let len = buf.len();

        // Read from removable region.
        let (len, bytes) = if *this.position < this.buffer.first_checkpoint() {
            let (bytes, popped) = this.buffer.pop(*this.position, len);
            *this.position += bytes;
            for (i, b) in popped.enumerate() {
                buf[i] = b;
            }
            (len - bytes, bytes)
        } else {
            (len, 0)
        };

        // Read from recycling region.
        let (len, bytes) = if *this.position < this.buffer.end() && len != 0 {
            let offset = bytes;
            let (bytes, popped) = this.buffer.pop_ref(*this.position, len);
            *this.position += bytes;
            for (i, b) in popped.copied().enumerate() {
                buf[offset + i] = b;
            }
            (len - bytes, offset + bytes)
        } else {
            (len, bytes)
        };

        // Pop from new reading region.
        if len != 0 {
            match this.inner.poll_read(cx, &mut buf[bytes..]) {
                Poll::Ready(Ok(b)) => {
                    *this.position += b;
                    if this.buffer.is_recording() {
                        this.buffer.push(&buf[bytes..]);
                    }
                    Poll::Ready(Ok(b))
                }
                p => p,
            }
        } else {
            Poll::Ready(Ok(bytes))
        }
    }
}

impl<R> Rewind for BufferInput<R> {
    type Checkpoint = usize;

    fn poll_position(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Poll::Ready(Ok(self.position))
    }

    fn poll_set_checkpoint(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        let this = self.project();
        this.buffer.set_checkpoint(*this.position);
        Poll::Ready(Ok(*this.position))
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        let this = self.project();
        this.buffer.rewind(checkpoint);
        *this.position = checkpoint;
        Poll::Ready(Ok(()))
    }
}
