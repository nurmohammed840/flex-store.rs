use crate::page_no::PageNo;
use crate::Pages;
use std::fs::File;
use std::io::Result;
use std::ops::{Deref, DerefMut};

use page_lock::LockGuard;
use tokio::task::spawn_blocking;

pub struct Page<'a, K: PageNo, const NBYTES: usize> {
    pub num:   u32,
    pub buf:   [u8; NBYTES],
    pub(crate) file:  &'static File,
    pub(crate) _lock: LockGuard<'a, K>,
}

impl<K: PageNo, const NBYTES: usize> Page<'_, K, NBYTES> {
    pub async fn write(self) -> Result<()> {
        let Self { file, num, buf, .. } = self;
        spawn_blocking(move || Pages::<K, NBYTES>::_write(file, num, buf))
            .await
            .unwrap()
    }
}

impl<'a, K: PageNo, const NBYTES: usize> Deref for Page<'a, K, NBYTES> {
    type Target = [u8; NBYTES];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}
impl<'a, K: PageNo, const NBYTES: usize> DerefMut for Page<'a, K, NBYTES> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}
