#![allow(warnings)]
#![feature(cursor_remaining)]
#![feature(generic_const_exprs)]

mod file;
mod free;
mod meta;
mod page;
mod page_no;

use core::num;
use file::FileExt;
// use free::Free;
use meta::Meta;
use page::Page;
use page_lock::PageLocker;
use page_no::PageNo;
use std::{
    fs::File,
    future::Future,
    io::{ErrorKind, Result},
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};
use tokio::task::spawn_blocking;

pub struct Pages<K, const NBYTES: usize>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    /// Total Page number
    len:    AtomicU32,
    file:   &'static File,
    meta:   Meta<K, NBYTES>,
    locker: PageLocker<K>,
    // free:   Free<K, NBYTES>,
}

impl<K, const NBYTES: usize> Pages<K, NBYTES>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut meta = Meta::<K, NBYTES>::new();
        let file = File::options().read(true).write(true).create(true).open(path)?;

        let file_len = file.metadata()?.len();
        // So that, There is no residue bytes.
        if file_len % NBYTES as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }
        // New File? Write metadata.
        if file_len == 0 {
            Self::write(&file, 0, meta.to_buf())?;
        }
        // Exist File? Read metadata.
        else {
            meta.update_from(Self::_read(&file, 0)?)?;
        }

        let file: &'static File = Box::leak(Box::new(file));
        // let free = Free::new(meta.free_list_tail, &file)?;
        Ok(Self {
            // free,
            meta,
            file,
            locker: PageLocker::new(),
            len: AtomicU32::new(file_len as u32 / NBYTES as u32),
        })
    }

    fn _read(file: &File, num: u32) -> Result<[u8; NBYTES]> {
        let mut buf = [0; NBYTES];
        file.read_exact_at(&mut buf, NBYTES as u64 * num as u64)?;
        Ok(buf)
    }

    fn write(file: &File, num: u32, buf: [u8; NBYTES]) -> Result<()> {
        file.write_all_at(&buf, NBYTES as u64 * num as u64)?;
        Ok(())
    }

    pub async fn read(&self, num: K) -> Result<[u8; NBYTES]> {
        self.locker.unlock(num).await;
        let num = num.as_u32();
        let file = self.file;
        spawn_blocking(move || Self::_read(file, num)).await.unwrap()
    }

    pub async fn goto(&self, num: K) -> Result<Page<'_, K, NBYTES>> {
        let _lock = self.locker.lock(num).await;
        let num = num.as_u32();
        let file = self.file;
        let buf = spawn_blocking(move || Self::_read(file, num)).await.unwrap()?;
        Ok(Page { _lock, file, num, buf })
    }

    // pub async fn create(&self) -> Result<Page<'_, K, NBYTES>> {
    //     let num = loop {
    //         let mut freelist = self.free.lock().unwrap();
    //         if let Some(num) = freelist.pop() {
    //             break num;
    //         }
    //         let prev = freelist.prev;
    //         if prev.as_u32() == 0 {
    //             break K::new(self.len.fetch_add(1, Ordering::SeqCst));
    //         }
    //         let free_list_tail = self.meta.free_list_tail();
    //         Self::write_async(self.file, K::new(free_list_tail), freelist.to_byte()).await?;
    //         freelist.update_from(Self::read_async(self.file, prev).await?);
    //     };
    //     Ok(self.goto(num).await)
    // }
}

impl<K, const NBYTES: usize> Drop for Pages<K, NBYTES>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    fn drop(&mut self) {
        // self.meta.free_list_tail = self.free.curr;
        Self::write(self.file, 0, self.meta.to_buf()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() { let _pages: Pages<u16, 4096> = Pages::open("test.db").unwrap(); }
}
