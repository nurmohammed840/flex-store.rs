An async library for locking page address.

You may want to use this library to lock spacific page address. (For example database)

#### Example

```rust
use page_lock::PageLocker;
use std::sync::Arc;
use std::time::Duration;
use tokio::{spawn, time::sleep};

let locker = Arc::new(PageLocker::new());
let locker1 = locker.clone();
tokio::try_join!(
    spawn(async move {
        let _lock = locker.lock(1).await;
        println!("(1) Page 1: Locked");
        sleep(Duration::from_secs(3)).await;
        println!("(3) Page 1: Droping lock");
    }),
    spawn(async move {
        println!("(2) Page 1: Waiting for unlock...");
        locker1.unlock(1).await;
        println!("(4) Page 1: Unlocked!");
    })
)
.unwrap();
```

### Locking mechanism

You may think something like this: `Map<Number, Mutex>`. where `Number` is page address.

Unlocking method: `unlock(Number)` does:

```
function Unlock(Number):
    if there is any `Mutex` in `Map` by `Number`:
        Wait for it, to get removed. 
    
    unlocked! 
```

locking method is `lock(Number)`,

```
function Lock(Number):
    If there is any `Mutex` in `Map` by `Number`:
        Wait for it, to get removed. 
    
    Add a `Mutex` to `Map` by `Number`,

    Return a `Lock` guard:
        When `Lock` is dropped, remove the `Mutex` from `Map` by `Number`.
```
