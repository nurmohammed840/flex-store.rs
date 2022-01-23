use crate::page_no::PageNo;
use crate::Pages;
use std::io::Result;
use std::ops::{Deref, DerefMut};

use page_lock::LockGuard;

pub struct Page<'a, K: PageNo, const NBYTES: usize> {
    pub buf: [u8; NBYTES],
    pub(crate) num: K,
    pub(crate) _lock: LockGuard<'a, K>,
    pub(crate) pages: &'a Pages<K, NBYTES>,
}

impl<K: PageNo, const NBYTES: usize> Page<'_, K, NBYTES> {
    pub async fn write(self) -> Result<usize> {
        self.pages.write(self.num, self.buf).await
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
