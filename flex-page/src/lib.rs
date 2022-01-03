#![allow(warnings)]
#![feature(cursor_remaining)]
#![feature(generic_const_exprs)]

mod file;
mod free;
mod locker;
mod meta;
mod page;
mod page_no;

use file::FileExt;
use free::FreeList;
use locker::Lockers;
use meta::Meta;
use page::Page;
use page_no::PageNo;
use std::{
    fs::File,
    future::Future,
    io::{ErrorKind, Result},
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

pub struct Pages<K, const NBYTES: usize>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    /// Total Page number
    len: AtomicU32,
    file: &'static File,
    lockers: Lockers<K>,
    meta: Meta<K, NBYTES>,
    free: Mutex<FreeList<K, NBYTES>>,
}

impl<K, const NBYTES: usize> Pages<K, NBYTES>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    pub fn open(path: &str) -> Result<Self> {
        let mut meta = Meta::<K, NBYTES>::new();
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let file_len = file.metadata()?.len();
        // So that, There is no residue bytes.
        if file_len % NBYTES as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }
        // New File? Write metadata.
        if file_len == 0 {
            file.write_all_at(&meta.to_bytes(), 0)?;
        }
        // Exist File? Read metadata.
        else {
            let mut bytes = [0; NBYTES];
            file.read_exact_at(&mut bytes, 0)?;
            meta.extend_from(bytes)?;
        }
        let buf = Self::read_sync(&file, meta.last_free_page().into())?;
        Ok(Self {
            meta,
            free: Mutex::new(FreeList::from(buf)),
            file: Box::leak(Box::new(file)),
            lockers: Lockers::new(),
            len: AtomicU32::new(file_len as u32 / NBYTES as u32),
        })
    }

    pub fn write_metadata(&self) -> Result<()> {
        self.file.write_all_at(&self.meta.to_bytes(), 0)?;
        Ok(())
    }

    fn read_sync(file: &File, num: u64) -> Result<[u8; NBYTES]> {
        let mut buf = [0; NBYTES];
        file.read_exact_at(&mut buf, NBYTES as u64 * num)?;
        Ok(buf)
    }

    async fn read_async(file: &'static File, num: K) -> Result<[u8; NBYTES]> {
        let num = num.as_u32() as u64;
        tokio::task::spawn_blocking(move || Self::read_sync(file, num))
            .await
            .unwrap()
    }

    pub async fn read(&self, num: K) -> Result<[u8; NBYTES]> {
        self.lockers.unlock(num).await;
        Self::read_async(self.file, num).await
    }

    pub async fn goto(&self, num: K) -> Page<'_, K, NBYTES> {
        Page {
            num,
            file: self.file,
            _lock: self.lockers.lock(num).await,
        }
    }

    pub async fn create(&self) -> Page<'_, K, NBYTES> {
        let mut num = K::new(0);

        // let num = loop {
        //     let mut list = self.free.lock().unwrap();
        //     if let Some(page) = list.pop() {
        //         break num;
        //     }
        //     list.prev;
        //     // num = self.len.fetch_add(1, Ordering::SeqCst) as K;
        //     break K::new(0);
        // };

        // num = K::new(self.len.fetch_add(1, Ordering::SeqCst));

        self.goto(num).await
    }
}

impl<K, const NBYTES: usize> Drop for Pages<K, NBYTES>
where
    K: PageNo,
    [(); K::SIZE]:,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    fn drop(&mut self) {
        let _ = self.file.write_all_at(&self.meta.to_bytes(), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let _pages: Pages<u16, 4096> = Pages::open("test.db").unwrap();
        _pages.write_metadata();
    }
}
