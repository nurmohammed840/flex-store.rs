#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![allow(clippy::len_without_is_empty)]

mod file;
mod page;
mod page_no;
mod size_info;

use file::FileExt;
use page::Page;
use page_no::PageNo;
use size_info::SizeInfo;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use page_lock::PageLocker;
use tokio::task::spawn_blocking;

macro_rules! err { [$cond: expr, $msg: expr] => { if $cond { return Err(Error::new(ErrorKind::InvalidInput, $msg)); } }; }

pub struct Pages<K: PageNo, const NBYTES: usize> {
    /// Total Page number
    len: AtomicU32,
    file: &'static File,
    locker: PageLocker<K>,
    size_info: SizeInfo,
}

impl<K: PageNo, const NBYTES: usize> Pages<K, NBYTES> {
    /// Create a new `Pages` instance.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        err!(NBYTES < 64, "Page size should >= 64 bytes");
        err!(NBYTES > 1024 * 256, "Page size should > 256 kilobytes");
        let size_info = SizeInfo { block_size: NBYTES as u32, pages_len_nbytes: K::SIZE as u8 };
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
        // Exist File?  Check size info.
        else {
            let mut buf = [0; 4];
            file.read_exact_at(&mut buf, 0)?;
            let info = SizeInfo::from(buf);
            err!(info != size_info, format!("Expected {:?}, but got: {:?}", info, size_info));
        }
        let len = AtomicU32::new(len);
        let file: &'static File = Box::leak(Box::new(file));
        Ok(Self { size_info, file, locker: PageLocker::new(), len })
    }

    pub async fn read(&self, num: K) -> Result<[u8; NBYTES]> {
        debug_assert!((1..self.len()).contains(&num.as_u32()));

        self.locker.unlock(num).await;
        let num = num.as_u32() as u64;
        let file = self.file;
        spawn_blocking(move || {
            let mut buf = [0; NBYTES];
            file.read_exact_at(&mut buf, NBYTES as u64 * num)?;
            Ok(buf)
        })
        .await
        .unwrap()
    }

    pub async fn goto(&self, num: K) -> Result<Page<'_, K, NBYTES>> {
        let buf = self.read(num).await?;
        let _lock = self.locker.lock(num);
        Ok(Page { _lock, num, buf, pages: self })
    }

    pub async fn write(&self, num: K, buf: [u8; NBYTES]) -> Result<usize> {
        debug_assert!((1..self.len()).contains(&num.as_u32()));

        let num = num.as_u32() as u64;
        let file = self.file;
        spawn_blocking(move || file.write_all_at(&buf, NBYTES as u64 * num)).await.unwrap()
    }

    pub async fn create(&self, buf: [u8; NBYTES]) -> Result<usize> {
        let num = self.len.fetch_add(1, Ordering::Relaxed);
        self.write(PageNo::new(num), buf).await
    }

    pub fn len(&self) -> u32 {
        self.len.load(Ordering::Relaxed)
    }
}

impl<K: PageNo, const NBYTES: usize> Drop for Pages<K, NBYTES> {
    fn drop(&mut self) {
        self.file.write_all_at(&self.size_info.to_bytes(), 0).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn init() -> Result<usize> {
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
        pages.create([0; 64]).await?;

        assert_eq!(pages.len(), 3);

        pages.write(2, [2; 64]).await?;
        assert_eq!(pages.len(), 3);
        assert_eq!(pages.read(2).await?, [2; 64]);
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
                std::fs::remove_file("test.db").unwrap()
            });
    }
}
