mod file;
mod locker;
mod meta;
mod page;
mod page_no;

use file::FileExt;
use locker::Lockers;
use meta::Metadata;
use page::Page;
use page_no::PageNo;
use std::{
    fs::File,
    io::{ErrorKind, Result},
};

pub struct Pages<K: PageNo, const NBYTES: usize> {
    /// Total Page number
    _len: K,
    file: &'static File,
    lockers: Lockers<K>,
    _metadata: Metadata<K, NBYTES>,
}

impl<K: PageNo, const NBYTES: usize> Pages<K, NBYTES> {
    pub fn open(path: &str) -> Result<Self> {
        let metadata = Metadata::<K, NBYTES>::new();

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

        let this = Self {
            _metadata: metadata,
            file: Box::leak(Box::new(file)),
            lockers: Lockers::new(),
            _len: K::new(file_len as usize / NBYTES),
        };
        // New File? Write default metadata.
        if file_len == 0 {}

        Ok(this)
    }

    fn _write_raw_meta() {}

    fn _read_raw_meta() -> Vec<u8> {
        let raw: Vec<u8> = Vec::with_capacity(NBYTES);
        let page_no = 0;
        loop {
            // let buf = pages.read_page(root_page_no)?;
            // // every mata pages contain first 4 byte as next page pointer.
            // root_page_no = u32::from_le_bytes(buf[..4].try_into().unwrap());
            // raw.extend_from_slice(&buf[4..]);
            if page_no == 0 {
                return raw;
            }
        }
    }

    fn read_sync(file: &File, num: u64) -> Result<[u8; NBYTES]> {
        let mut buf = [0; NBYTES];
        file.read_exact_at(&mut buf, NBYTES as u64 * num)?;
        Ok(buf)
    }

    async fn read_unchecked(file: &'static File, num: K) -> Result<[u8; NBYTES]> {
        let num = num.as_u32() as u64;
        tokio::task::spawn_blocking(move || Self::read_sync(file, num))
            .await
            .unwrap()
    }

    pub async fn read(&self, num: K) -> Result<[u8; NBYTES]> {
        self.lockers.unlock(num).await;
        Self::read_unchecked(self.file, num).await
    }

    pub async fn goto(&self, num: K) -> Page<'_, K, NBYTES> {
        Page {
            num,
            file: self.file,
            _lock: self.lockers.lock(num).await,
        }
    }
}
