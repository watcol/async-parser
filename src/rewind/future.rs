use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Rewind;
use crate::Result;

#[derive(Debug)]
pub struct PositionFuture<'a, R: ?Sized>(&'a mut R);

impl<'a, R: ?Sized> PositionFuture<'a, R> {
    pub(super) fn new(inner: &'a mut R) -> Self {
        Self(inner)
    }
}

impl<R: Unpin + ?Sized> Unpin for PositionFuture<'_, R> {}

impl<R: Rewind + Unpin + ?Sized> Future for PositionFuture<'_, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_position(cx)
    }
}

#[derive(Debug)]
pub struct SetCheckpointFuture<'a, R: ?Sized>(&'a mut R);

impl<'a, R: ?Sized> SetCheckpointFuture<'a, R> {
    pub(super) fn new(inner: &'a mut R) -> Self {
        Self(inner)
    }
}

impl<R: Unpin + ?Sized> Unpin for SetCheckpointFuture<'_, R> {}

impl<R: Rewind + Unpin + ?Sized> Future for SetCheckpointFuture<'_, R> {
    type Output = Result<R::Checkpoint>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_set_checkpoint(cx)
    }
}

#[derive(Debug)]
pub struct RewindFuture<'a, R: Rewind + ?Sized>(&'a mut R, R::Checkpoint);

impl<'a, R: Rewind + ?Sized> RewindFuture<'a, R> {
    pub(super) fn new(inner: &'a mut R, checkpoint: R::Checkpoint) -> Self {
        Self(inner, checkpoint)
    }
}

impl<R: Rewind + Unpin + ?Sized> Unpin for RewindFuture<'_, R> {}

impl<R: Rewind + Unpin + ?Sized> Future for RewindFuture<'_, R>
where
    R::Checkpoint: Clone,
{
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let ck = self.1.clone();
        Pin::new(&mut *self.0).poll_rewind(cx, ck)
    }
}
