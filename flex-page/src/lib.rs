#![allow(warnings)]

// use std::{fs::File, io::*};
use tokio::{fs, io::*};

use meta::Metadata;
use page_no::PageNo;

mod meta;
mod page;
mod page_no;

pub struct Pages<P: PageNo, const PS: usize> {
    file: fs::File,
    /// Total Page number
    len: P,
    metadata: Metadata<P, PS>,
}

impl<P: PageNo, const PS: usize> Pages<P, PS> {
    async fn open(path: &str) -> Result<Self> {
        let metadata = Metadata::<P, PS>::new();

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;

        let file_len = file.metadata().await?.len();
        // So that, There is no residue bytes.
        if file_len % PS as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut this = Self {
            file,
            metadata,
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
}

impl<P: PageNo, const PAGE_SIZE: usize> Drop for Pages<P, PAGE_SIZE> {
    fn drop(&mut self) {
        todo!()
    }
}
