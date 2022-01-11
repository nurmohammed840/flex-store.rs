use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};

type Locker<T> = RwLock<HashMap<T, Vec<Waker>>>;

pub struct UnLock<'a, T> {
    num:    T,
    state:  bool,
    locker: &'a Locker<T>,
}

impl<'a, T: Unpin + Eq + Hash> Future for UnLock<'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state {
            return Poll::Ready(());
        }
        let this = self.get_mut();
        this.state = true;
        this.locker.write().unwrap().get_mut(&this.num).unwrap().push(cx.waker().clone());
        Poll::Pending
    }
}

pub struct LockGuard<'a, T: Eq + Hash> {
    num:    T,
    locker: &'a Locker<T>,
}

impl<T: Eq + Hash> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        for waker in self.locker.write().unwrap().remove(&self.num).unwrap() {
            waker.wake();
        }
    }
}

#[derive(Default)]
pub struct PageLocker<T> {
    locker: Locker<T>,
}

impl<T: Eq + Hash + Copy + Unpin> PageLocker<T> {
    pub fn new() -> Self { Self { locker: RwLock::new(HashMap::new()) } }

    #[inline(always)]
    pub fn unlock(&self, num: T) -> UnLock<T> {
        UnLock { state: self.locker.read().unwrap().get(&num).is_none(), locker: &self.locker, num }
    }

    #[inline(always)]
    pub async fn lock(&self, num: T) -> LockGuard<'_, T> {
        self.unlock(num).await;
        self.locker.write().unwrap().insert(num, Vec::new());
        LockGuard { num, locker: &self.locker }
    }
}
