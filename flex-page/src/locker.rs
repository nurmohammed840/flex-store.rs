use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll, Waker},
};

type LockersMap<K> = RwLock<HashMap<K, Vec<Waker>>>;

pub struct Lock<'a, K: Eq + Hash> {
    no: K,
    lockers: &'a LockersMap<K>,
}
impl<'a, K: Eq + Hash + Clone> Lock<'a, K> {
    fn new(lockers: &'a LockersMap<K>, no: K) -> Self {
        lockers.write().unwrap().insert(no.clone(), Vec::new());
        Self { no, lockers }
    }
}
impl<K: Eq + Hash> Drop for Lock<'_, K> {
    fn drop(&mut self) {
        for waker in self.lockers.write().unwrap().remove(&self.no).unwrap() {
            waker.wake();
        }
    }
}

pub struct UnLock<'a, K> {
    no: K,
    state: bool,
    lockers: &'a LockersMap<K>,
    data: Option<()>,
}
impl<'a, K: Unpin + Eq + Hash> Future for UnLock<'a, K> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state {
            return Poll::Ready(());
        }
        let this = self.get_mut();
        this.state = true;
        {
            let mut lockers = this.lockers.write().unwrap();
            let wakers = lockers.get_mut(&this.no).unwrap();
            wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }
}


pub struct Lockers<K>(LockersMap<K>);
impl<K: Eq + Hash + Clone + Unpin> Lockers<K> {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn unlock(&self, no: K) -> UnLock<K> {
        UnLock {
            state: self.0.read().unwrap().get(&no).is_none(),
            lockers: &self.0,
            data: None,
            no,
        }
    }
    pub async fn lock(&self, no: K) -> Lock<'_, K> {
        self.unlock(no.clone()).await;
        Lock::new(&self.0, no)
    }
}

#[cfg(test)]
mod tests {
    use super::Lockers;
    use std::{
        sync::{
            atomic::{AtomicU8, Ordering},
            Arc,
        },
        time::Duration,
    };
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn basic() {
        let lockers = Lockers::new();
        lockers.unlock(0).await;
        lockers.unlock(0).await;
        let locker = lockers.lock(0).await;
        drop(locker);
        lockers.unlock(0).await;
    }

    #[tokio::test]
    async fn deadlock() {
        let join1 = tokio::spawn(async {
            let lockers = Lockers::new();
            let _locker = lockers.lock(0).await;
            let _locker = lockers.lock(0).await;
            panic!("Never reached");
        });
        timeout(Duration::from_secs(1), join1).await.err().unwrap();
    }

    #[tokio::test]
    async fn lock() {
        fn step(n: u8) {
            static C: AtomicU8 = AtomicU8::new(0);
            assert_eq!(C.fetch_add(1, Ordering::SeqCst), n)
        }
        let _lockers = Arc::new(Lockers::new());
        let lockers = _lockers.clone();
        let t1 = tokio::spawn(async move {
            step(0);
            let locker = lockers.lock(0).await;
            sleep(Duration::from_millis(10)).await;
            step(2);
            drop(locker);
        });
        let lockers = _lockers.clone();
        let t2 = tokio::spawn(async move {
            step(1);
            lockers.unlock(0).await;
            step(3);
        });
        tokio::join!(t1, t2);
    }
}