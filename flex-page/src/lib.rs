#![allow(warnings)]

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
    collections::HashMap,
    fs::File,
    future::Future,
    io::{ErrorKind, Result},
    sync::{Arc, Mutex, RwLock},
    task::Waker,
};
use tokio::task;

pub struct Pages<P: PageNo, const PS: usize> {
    /// Total Page number
    len: P,
    file: &'static File,
    metadata: Metadata<P, PS>,
    lockers: Lockers<P>,
}

impl<P: PageNo, const PS: usize> Pages<P, PS> {
    fn open(path: &str) -> Result<Self> {
        let metadata = Metadata::<P, PS>::new();

        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let file_len = file.metadata()?.len();
        // So that, There is no residue bytes.
        if file_len % PS as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let this = Self {
            metadata,
            file: Box::leak(Box::new(file)),
            lockers: Lockers::new(),
            len: P::new(file_len as usize / PS),
        };
        // New File? Write default metadata.
        if file_len == 0 {}

        Ok(this)
    }

    fn _write_raw_meta() {}

    fn _read_raw_meta() -> Vec<u8> {
        let mut raw: Vec<u8> = Vec::with_capacity(PS);
        let mut page_no = 0;
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

    async fn read(&self, n: P) -> Result<[u8; PS]> {
        self.lockers.unlock(n).await;
        let file = self.file;
        // let num: u32 = n.try_into().unwrap();
        task::spawn_blocking(move || {
            let mut buf = [0; PS];
            // file.read(&mut buf, PS as u64 * n.try_into().unwrap() as u64)?;
            Ok(buf)
        })
        .await
        .unwrap()
    }

    // async fn get(&self, no: P) -> Page<'_, P, PS> {
    //     Page {
    //         no,
    //         lock: self.lockers.lock(no).await,
    //         file: self.file,
    //         buf: [0; PS],
    //     }
    // }

    // async fn lock(&self, no: &P) {
    //     let state = self.lockers.read().unwrap().get(no).is_none();
    //     let mut lockers = self.lockers.write().unwrap();
    //     let lock = WriteLock {
    //         state,
    //         wakers: lockers.get_mut(no),
    //     };
    //     lock.await;
    // }

    // async fn _write(&self, page_no: u32, buf: [u8; PS]) -> Result<()> {
    //     let file = self.file;
    //     task::spawn_blocking(move || {
    //         file.write(&buf, PS as u64 * page_no as u64)?;
    //         Ok(())
    //     })
    //     .await
    //     .unwrap()
    // }

    // fn read(&self, page_no: P) -> Result<[u8; PS]> {
    //     let lockers = self.lockers.lock().unwrap();
    //     match lockers.get(&page_no) {
    //         Some(page) => {}
    //         None => {}
    //     }
    //     // lockers

    //     // let page_no = page_no.try_into().unwrap();
    //     // self._read(page_no);

    //     todo!()
    // }
}
