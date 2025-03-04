//! Adapted from Titanix
extern crate alloc;

use alloc::{boxed::Box, sync::Arc, task::Wake, vec::Vec};
use core::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{ready, Context, Poll, Waker},
};

/// Get the waker of the current future.
#[inline(always)]
pub async fn get_waker() -> Waker {
    TakeWakerFuture.await
}

struct TakeWakerFuture;

impl Future for TakeWakerFuture {
    type Output = Waker;
    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(cx.waker().clone())
    }
}

/// A wrapper for a data structure that be sent between threads
pub struct SendWrapper<T>(pub T);

impl<T> SendWrapper<T> {
    pub fn new(data: T) -> Self {
        SendWrapper(data)
    }
}

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T: Deref> Deref for SendWrapper<T> {
    type Target = T::Target;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: DerefMut> DerefMut for SendWrapper<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

/// A waker that wakes up the current thread when called.
struct BlockWaker;

impl Wake for BlockWaker {
    fn wake(self: Arc<Self>) {
        log::trace!("block waker wakes");
    }
}

/// Run a future to completion on the current thread.
/// Note that since this function is used in kernel mode,
/// we won't switch thread when the inner future pending.
/// Instead, we just poll the inner future again and again.
#[allow(dead_code)]
pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
    // Pin the future so it can be polled.
    let mut fut = Box::pin(fut);

    let waker = Arc::new(BlockWaker).into();
    let mut cx = Context::from_waker(&waker);

    // Run the future to completion.
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => return res,
            Poll::Pending => continue,
        }
    }
}

pub enum SelectOutput<T1, T2> {
    Output1(T1),
    Output2(T2),
}

/// Select two futures at a time.
/// Note that future1 has a higher level than future2
pub struct Select2Futures<T1, T2, F1, F2>
where
    F1: Future<Output = T1>,
    F2: Future<Output = T2>,
{
    future1: F1,
    future2: F2,
}

impl<T1, T2, F1, F2> Select2Futures<T1, T2, F1, F2>
where
    F1: Future<Output = T1>,
    F2: Future<Output = T2>,
{
    #[allow(dead_code)]
    pub fn new(future1: F1, future2: F2) -> Self {
        Self { future1, future2 }
    }
}

impl<T1, T2, F1, F2> Future for Select2Futures<T1, T2, F1, F2>
where
    F1: Future<Output = T1>,
    F2: Future<Output = T2>,
{
    type Output = SelectOutput<T1, T2>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let ret = unsafe { Pin::new_unchecked(&mut this.future1).poll(cx) };
        if ret.is_ready() {
            return Poll::Ready(SelectOutput::Output1(ready!(ret)));
        }
        let ret = unsafe { Pin::new_unchecked(&mut this.future2).poll(cx) };
        if ret.is_ready() {
            return Poll::Ready(SelectOutput::Output2(ready!(ret)));
        }
        Poll::Pending
    }
}

pub struct AnyFuture<'a, T> {
    futures: Vec<Async<'a, T>>,
    has_returned: bool,
}
#[allow(dead_code)]
impl<'a, T> AnyFuture<'a, T> {
    pub fn new() -> Self {
        Self {
            futures: Vec::new(),
            has_returned: false,
        }
    }
    pub fn push(&mut self, future: Async<'a, T>) {
        self.futures.push(future);
    }

    pub fn new_with(futures: Vec<Async<'a, T>>) -> Self {
        debug_assert!(futures.len() > 0);
        Self {
            futures,
            has_returned: false,
        }
    }
}

impl<T> Future for AnyFuture<'_, T> {
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        if this.has_returned {
            return Poll::Pending;
        }

        for (i, future) in this.futures.iter_mut().enumerate() {
            let result = unsafe { Pin::new_unchecked(future).poll(cx) };
            if let Poll::Ready(ret) = result {
                this.has_returned = true;
                return Poll::Ready((i, ret));
            }
        }

        Poll::Pending
    }
}

struct YieldFuture {
    has_yielded: bool,
}

impl YieldFuture {
    const fn new() -> Self {
        Self { has_yielded: false }
    }
}

impl Future for YieldFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.has_yielded {
            true => Poll::Ready(()),
            false => {
                self.has_yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

/// Yield the current thread (the scheduler will switch to the next thread)
pub async fn yield_now() {
    //log::info!("yield_now");
    YieldFuture::new().await;
}

struct SuspendFuture {
    has_suspended: bool,
}
#[allow(dead_code)]
impl SuspendFuture {
    const fn new() -> Self {
        Self {
            has_suspended: false,
        }
    }
}

impl Future for SuspendFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.has_suspended {
            true => Poll::Ready(()),
            false => {
                self.has_suspended = true;
                Poll::Pending
            }
        }
    }
}
#[allow(dead_code)]
pub async fn suspend_now() {
    SuspendFuture::new().await
}

pub type Async<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// create an `Async<T::Output>` from a future, usually an async block.
/// A typical usage is like this:
/// ```
/// fn stat(&self) -> ASysResult<NodeStat> {
///     dyn_future(async {
///         let f = self.lock.lock().await.stat();
///         f.await
///     })
/// }
/// ```
#[allow(dead_code)]
pub fn dyn_future<'a, T: Future + Send + 'a>(async_blk: T) -> Async<'a, T::Output> {
    Box::pin(async_blk)
}
