use crate::page_no::PageNo;
use crate::Pages;
use std::fs::File;
use std::io::Result;

use page_lock::LockGuard;
use tokio::task::spawn_blocking;

pub struct Page<'a, K: PageNo, const NBYTES: usize> {
    pub num:   u32,
    pub buf:   [u8; NBYTES],
    pub file:  &'static File,
    pub _lock: LockGuard<'a, K>,
}

impl<K: PageNo, const NBYTES: usize> Page<'_, K, NBYTES> {
    pub async fn write(self) -> Result<()> {
        let Self { file, num, buf, .. } = self;
        spawn_blocking(move || Pages::<K, NBYTES>::_write(file, num, buf))
            .await
            .unwrap()
    }
}
