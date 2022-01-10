use tokio::task::spawn_blocking;

use crate::{file::FileExt, locker::Lock, page_no::PageNo, Pages};
use std::{fs::File, future::Future, io::Result, ops::Deref};

pub struct Page<'a, K: PageNo, const NBYTES: usize> {
    pub num: u32,
    pub file: &'static File,
    pub buf: [u8; NBYTES],
    pub _lock: Lock<'a, K>,
}

impl<K, const NBYTES: usize> Page<'_, K, NBYTES>
where
    K: PageNo,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    pub async fn write(self) -> Result<usize> {
        spawn_blocking(move || Pages::<K, NBYTES>::write(self.file, self.num, self.buf))
            .await
            .unwrap()
    }
}
