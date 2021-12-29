use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};

type LockersMap<K> = RwLock<HashMap<K, Vec<Waker>>>;

pub struct Lock<'a, K: Eq + Hash> {
    num: K,
    lockers: &'a LockersMap<K>,
}
impl<'a, K: Eq + Hash + Clone> Lock<'a, K> {
    fn new(lockers: &'a LockersMap<K>, num: K) -> Self {
        lockers.write().unwrap().insert(num.clone(), Vec::new());
        Self { num, lockers }
    }
}
impl<K: Eq + Hash> Drop for Lock<'_, K> {
    fn drop(&mut self) {
        for waker in self.lockers.write().unwrap().remove(&self.num).unwrap() {
            waker.wake();
        }
    }
}

pub struct UnLock<'a, K> {
    num: K,
    state: bool,
    lockers: &'a LockersMap<K>,
}
impl<'a, K: Unpin + Eq + Hash> Future for UnLock<'a, K> {
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

pub struct Lockers<K>(LockersMap<K>);
impl<K: Eq + Hash + Clone + Unpin> Lockers<K> {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn unlock(&self, num: K) -> UnLock<K> {
        UnLock {
            state: self.0.read().unwrap().get(&num).is_none(),
            lockers: &self.0,
            num,
        }
    }
    pub async fn lock(&self, num: K) -> Lock<'_, K> {
        self.unlock(num.clone()).await;
        Lock::new(&self.0, num)
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
        let _ = tokio::join!(t1, t2);
    }
}
