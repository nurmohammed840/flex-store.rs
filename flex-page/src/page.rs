use crate::{file::FileExt, locker::Lock, page_no::PageNo, Pages};
use std::{fs::File, future::Future, io::Result};

pub struct Page<'a, K: PageNo, const NBYTES: usize> {
    pub num: K,
    pub file: &'static File,
    pub _lock: Lock<'a, K>,
}

impl<K: PageNo, const NBYTES: usize> Page<'_, K, NBYTES> {
    pub fn read(&self) -> impl Future<Output = Result<[u8; NBYTES]>> {
        Pages::<K, NBYTES>::read_unchecked(self.file, self.num)
    }
    pub async fn write(self, buf: [u8; NBYTES]) -> Result<usize> {
        let file = self.file;
        let num = self.num.as_u32() as u64;
        tokio::task::spawn_blocking(move || file.write_all_at(&buf, NBYTES as u64 * num))
            .await
            .unwrap()
    }
}
