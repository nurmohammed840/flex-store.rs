//! An async library for locking page address.
//! You may want to use this library to lock spacific page address. (For example database)
//! 
//! ### Example
//! ```
//! use page_lock::PageLocker;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use tokio::{spawn, time::sleep};
//!
//! let locker = Arc::new(PageLocker::new());
//! let locker1 = locker.clone();
//! tokio::try_join!(
//!     spawn(async move {
//!         let _lock = locker.lock(1).await;
//!         println!("(1) Page 1: Locked");
//!         sleep(Duration::from_secs(3)).await;
//!         println!("(3) Page 1: Droping lock");
//!     }),
//!     spawn(async move {
//!         println!("(2) Page 1: Waiting for unlock...");
//!         locker1.unlock(1).await;
//!         println!("(4) Page 1: Unlocked!");
//!     })
//! )
//! .unwrap();
//! ```

use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};


type Locker<T> = RwLock<HashMap<T, Vec<Waker>>>;

pub struct Lock<'a, T: Eq + Hash> {
    num: T,
    locker: &'a Locker<T>,
}

impl<'a, T: Eq + Hash + Clone> Lock<'a, T> {
    fn new(locker: &'a Locker<T>, num: T) -> Self {
        locker.write().unwrap().insert(num.clone(), Vec::new());
        Self { num, locker }
    }
}

impl<T: Eq + Hash> Drop for Lock<'_, T> {
    fn drop(&mut self) {
        for waker in self.locker.write().unwrap().remove(&self.num).unwrap() {
            waker.wake();
        }
    }
}

pub struct UnLock<'a, T> {
    num: T,
    state: bool,
    lockers: &'a Locker<T>,
}

impl<'a, T: Unpin + Eq + Hash> Future for UnLock<'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state {
            return Poll::Ready(());
        }
        let this = self.get_mut();
        this.state = true;
        this.lockers
            .write()
            .unwrap()
            .get_mut(&this.num)
            .unwrap()
            .push(cx.waker().clone());

        Poll::Pending
    }
}

pub struct PageLocker<T> {
    locker: Locker<T>,
}

impl<T: Eq + Hash + Clone + Unpin> PageLocker<T> {
    pub fn new() -> Self {
        Self {
            locker: RwLock::new(HashMap::new()),
        }
    }

    pub fn unlock(&self, num: T) -> UnLock<T> {
        UnLock {
            state: self.locker.read().unwrap().get(&num).is_none(),
            lockers: &self.locker,
            num,
        }
    }

    pub async fn lock(&self, num: T) -> Lock<'_, T> {
        self.unlock(num.clone()).await;
        Lock::new(&self.locker, num)
    }
}

#[cfg(test)]
mod tests {
    use super::PageLocker;
    use std::time::Duration;
    use tokio::spawn;
    use tokio::time::timeout;

    #[tokio::test]
    async fn basic() {
        let lockers = PageLocker::new();
        lockers.unlock(0).await;
        lockers.unlock(0).await;
        let locker = lockers.lock(0).await;
        drop(locker);
        lockers.unlock(0).await;
    }

    #[tokio::test]
    async fn deadlock() {
        let join1 = spawn(async {
            let lockers = PageLocker::new();
            let _locker = lockers.lock(0).await;
            let _locker = lockers.lock(0).await;
            panic!("Never reached");
        });
        timeout(Duration::from_secs(1), join1).await.err().unwrap();
    }
}
