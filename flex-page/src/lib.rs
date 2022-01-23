#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod file;
mod meta;
mod page;
mod page_no;

use file::FileExt;
use meta::Meta;
use page::Page;
use page_no::PageNo;
use std::fs::File;
use std::io::{ErrorKind, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use page_lock::PageLocker;
use tokio::task::spawn_blocking;

pub struct Pages<K: PageNo, const NBYTES: usize> {
    /// Total Page number
    len:    AtomicU32,
    file:   &'static File,
    locker: PageLocker<K>,
    meta:   Meta<K, NBYTES>,
}

impl<K: PageNo, const NBYTES: usize> Pages<K, NBYTES> {
    /// Create a new `Pages` instance.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut meta = Meta::<K, NBYTES>::new()?;
        let file = File::options().read(true).write(true).create(true).open(path)?;
        let file_len = file.metadata()?.len();
        // So that, There is no residue bytes.
        if file_len % NBYTES as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut len = file_len as u32 / NBYTES as u32;
        // Is new file? set the length to 1.
        if file_len == 0 {
            len = 1;
        }
        // Exist File? Read metadata.
        else {
            meta.extend_from(Self::_read(&file, 0)?)?;
        }
        let len = AtomicU32::new(len);
        let file: &'static File = Box::leak(Box::new(file));
        Ok(Self { meta, file, locker: PageLocker::new(), len })
    }

    fn _read(file: &File, num: u32) -> Result<[u8; NBYTES]> {
        let mut buf = [0; NBYTES];
        file.read_exact_at(&mut buf, NBYTES as u64 * num as u64)?;
        Ok(buf)
    }

    fn _write(file: &File, num: u32, buf: [u8; NBYTES]) -> Result<()> {
        file.write_all_at(&buf, NBYTES as u64 * num as u64)?;
        Ok(())
    }

    pub async fn read(&self, num: K) -> Result<[u8; NBYTES]> {
        debug_assert!((1..self.len()).contains(&num.as_u32()));

        self.locker.unlock(num).await;
        let num = num.as_u32();
        let file = self.file;
        spawn_blocking(move || Self::_read(file, num)).await.unwrap()
    }

    pub async fn write(&self, num: K, buf: [u8; NBYTES]) -> Result<()> {
        debug_assert!((1..self.len()).contains(&num.as_u32()));
      
        let num = num.as_u32();
        let file = self.file;
        spawn_blocking(move || Self::_write(file, num, buf)).await.unwrap()
    }

    pub async fn goto(&self, num: K) -> Result<Page<'_, K, NBYTES>> {
        debug_assert!((1..self.len()).contains(&num.as_u32()));

        let _lock = self.locker.lock(num).await;
        let num = num.as_u32();
        let file = self.file;
        let buf = spawn_blocking(move || Self::_read(file, num)).await.unwrap()?;
        Ok(Page { _lock, file, num, buf })
    }

    pub async fn create(&self, buf: [u8; NBYTES]) -> Result<()> {
        let num = self.len.fetch_add(1, Ordering::Relaxed);
        self.write(PageNo::new(num), buf).await
    }

    pub fn len(&self) -> u32 {
        self.len.load(Ordering::Relaxed)
    }
}

impl<K: PageNo, const NBYTES: usize> Drop for Pages<K, NBYTES> {
    fn drop(&mut self) {
        Self::_write(self.file, 0, self.meta.to_bytes()).unwrap();
    }
}

#[test]
fn test_name() {
    let a = 1..2;
    println!("{}", a.contains(&0));
    println!("{}", a.contains(&2));
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::remove_file;

    async fn init() -> Result<()> {
        let pages = Pages::<u16, 64>::open("test.db")?;

        assert_eq!(pages.len(), 1);
        pages.create([0; 64]).await?;
        assert_eq!(pages.len(), 2);

        let mut page = pages.goto(1).await?;
        page.buf = [1; 64];
        page.write().await
    }

    async fn create_page() -> Result<()> {
        let pages = Pages::<u16, 64>::open("test.db")?;
        assert_eq!(pages.len(), 2);
        assert_eq!(pages.read(1).await?, [1; 64]);

        pages.create([2; 64]).await?;
        assert_eq!(pages.len(), 3);
        
        Ok(())
    }

    #[test]
    fn all() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                init().await.expect("Failed to Initialize");
                create_page().await.expect("Failed creating page");
                remove_file("test.db").unwrap()
            });
    }
}
