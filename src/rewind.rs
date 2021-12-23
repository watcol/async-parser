//! Rewinding for the input type.

mod buffer;
mod seek;

mod future;

pub use buffer::BufferInput;
pub use seek::SeekInput;

use future::{PositionFuture, RewindFuture, SetCheckpointFuture};

use crate::Result;
use futures::task::{Context, Poll};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

/// Rewinding the input, used by `peek()`, `choice()`, etc.
pub trait Rewind {
    /// A checkpoint to mark rewinding point.
    type Checkpoint;

    /// Getting current position.
    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>>;

    /// Marking a checkpoint.
    fn poll_set_checkpoint(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>>;

    /// Rewinding to the checkpoint.
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>>;

    /// An asynchronous version of `poll_position()`.
    fn position(&mut self) -> PositionFuture<'_, Self>
    where
        Self: Unpin,
    {
        PositionFuture::new(self)
    }

    /// An asynchronous version of `poll_set_checkpoint()`.
    fn set_checkpoint(&mut self) -> SetCheckpointFuture<'_, Self>
    where
        Self: Unpin,
    {
        SetCheckpointFuture::new(self)
    }

    /// An asynchronous version of `poll_rewind()`.
    fn rewind(&mut self, checkpoint: Self::Checkpoint) -> RewindFuture<'_, Self>
    where
        Self: Unpin,
        Self::Checkpoint: Clone,
    {
        RewindFuture::new(self, checkpoint)
    }
}

impl<T: Rewind + Unpin + ?Sized> Rewind for &mut T {
    type Checkpoint = T::Checkpoint;

    fn poll_position(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Pin::new(&mut **self).poll_position(cx)
    }

    fn poll_set_checkpoint(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        Pin::new(&mut **self).poll_set_checkpoint(cx)
    }

    fn poll_rewind(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        Pin::new(&mut **self).poll_rewind(cx, checkpoint)
    }
}

impl<T: Rewind + Unpin + ?Sized> Rewind for Box<T> {
    type Checkpoint = T::Checkpoint;

    fn poll_position(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Pin::new(&mut **self).poll_position(cx)
    }

    fn poll_set_checkpoint(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        Pin::new(&mut **self).poll_set_checkpoint(cx)
    }

    fn poll_rewind(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        Pin::new(&mut **self).poll_rewind(cx, checkpoint)
    }
}

impl<P> Rewind for Pin<P>
where
    P: DerefMut + Unpin,
    <P as Deref>::Target: Rewind,
{
    type Checkpoint = <<P as Deref>::Target as Rewind>::Checkpoint;

    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_position(cx)
    }

    fn poll_set_checkpoint(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Checkpoint>> {
        self.get_mut().as_mut().poll_set_checkpoint(cx)
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        checkpoint: Self::Checkpoint,
    ) -> Poll<Result<()>> {
        self.get_mut().as_mut().poll_rewind(cx, checkpoint)
    }
}
