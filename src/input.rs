use crate::{rewind::Rewind, ParseError, Result};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{AsyncRead, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// The input type for parsers, extending `AsyncRead` and implements `Stream`
    /// (and `TryStream`).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Input<R> {
        #[pin]
        inner: R,
    }
}

impl<T: AsyncRead> Stream for Input<T> {
    type Item = Result<u8>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut buf = [0; 1];
        match this.inner.poll_read(cx, &mut buf) {
            Poll::Ready(Ok(0)) => Poll::Ready(None),
            Poll::Ready(Ok(_)) => Poll::Ready(Some(Ok(buf[0]))),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(ParseError::ReadError { inner: e }))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncRead> AsyncRead for Input<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<T> From<T> for Input<T> {
    fn from(from: T) -> Self {
        Self { inner: from }
    }
}

impl<T: Rewind> Rewind for Input<T> {
    type Checkpoint = T::Checkpoint;

    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        self.project().inner.poll_position(cx)
    }

    fn poll_set_checkpoint(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        self.project().inner.poll_set_checkpoint(cx)
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        self.project().inner.poll_rewind(cx, checkpoint)
    }
}
