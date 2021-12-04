#![allow(warnings)]

// 0 indicate last page.

mod meta;
mod page_no;

use byte_seeker::{BytesReader, LittleEndian};
use meta::{Metadata, SizeInfo};
pub use page_no::*;

use std::{collections::HashMap, fs::File, io::*, marker::PhantomData};

pub struct Pages<P, const PS: usize, const PAGE_SIZE: usize> {
    file: File,
    metadata: [u8; PAGE_SIZE],
    _metadata: Metadata,
    _marker: PhantomData<P>,
}

#[test]
fn test_name() {
    let vec: Vec<u16> = (0..50).collect();
    for ele in vec.chunks(8) {
        println!("{:#?}", ele);
    }
}

// 0
// [1 2] [3 4] [5 6] [7]

impl<P, const PS: usize, const PAGE_SIZE: usize> Pages<P, PS, PAGE_SIZE>
where
    P: PageNo<PS>,
{
    fn write_meta(&mut self, data: Vec<u8>) -> Result<()> {
        let mut page_no = 0;
        let mut iter = data.chunks_exact(PAGE_SIZE);
        loop {
            let buf = self._read(page_no)?;
            page_no = u32::from_le_bytes(buf[..4].try_into().unwrap());
            match iter.next() {
                Some(data) => {
                    if page_no == 0 {
                        // create new.
                    }
                    self._write(page_no, data)?;
                }
                None => {}
            }
        }
        // for chunk in iter {
        //     let buf = self._read(page_no)?;
        //     page_no = u32::from_le_bytes(buf[..4].try_into().unwrap());
        //     self._write(page_no, &buf)?;
        // }
        return Ok(());
    }

    /// - Every meta page contain first 4 bytes (u32) as next page pointer,
    /// - It read raw data from those meta pages, Until there are no pages left to read...
    /// - It has no idea about the deta.
    fn read_meta(&mut self) -> Result<Vec<u8>> {
        let mut raw_data: Vec<u8> = Vec::with_capacity(PAGE_SIZE);
        let mut page_no = 0;
        loop {
            let buf = self._read(page_no)?;
            page_no = u32::from_le_bytes(buf[..4].try_into().unwrap());
            raw_data.extend_from_slice(&buf[4..]);
            if page_no == 0 {
                return Ok(raw_data);
            }
        }
    }

    pub fn open(path: &str) -> Result<Self> {
        let _metadata = Metadata::new(PS, PAGE_SIZE);

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let mut file = File::create(path)?;
                // file.write(&[0u32.to_le_bytes(), size_info.to_buf()].concat())?;
                file
            }
            Err(err) => return Err(err),
        };

        if file.metadata()?.len() % PAGE_SIZE as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut pages = Self {
            file,
            _metadata,
            metadata: [0; PAGE_SIZE],
            _marker: PhantomData,
        };

        let mut raw_mata: Vec<u8> = Vec::with_capacity(PAGE_SIZE);
        let mut mata_page_no = 0;
        loop {
            let buf = pages._read(mata_page_no)?;
            // every mata pages contain first 4 byte as next page pointer.
            mata_page_no = u32::from_le_bytes(buf[..4].try_into().unwrap());
            raw_mata.extend_from_slice(&buf[4..]);
            if mata_page_no == 0 {
                break;
            }
        }

        Ok(pages)
    }

    pub fn metadata(&mut self) -> &mut [u8] {
        &mut self.metadata[1..]
    }

    pub fn sync_metadata(&mut self) -> Result<()> {
        let buf = self.metadata;
        self.write(0, &buf)?;
        Ok(())
    }

    /// Clear Everything, Expect Matadata
    pub fn clear(&mut self) -> Result<()> {
        self.file.set_len(PAGE_SIZE as u64)?;
        Ok(())
    }

    fn _seek(&mut self, page_no: u32) -> Result<u64> {
        let pos = SeekFrom::Start(PAGE_SIZE as u64 * page_no as u64);
        self.file.seek(pos)
    }

    fn _write(&mut self, page_no: u32, buf: &[u8]) -> Result<usize> {
        self._seek(page_no)?;
        self.file.write(buf)
    }

    fn _read(&mut self, page_no: u32) -> Result<[u8; PAGE_SIZE]> {
        let mut buf = [0; PAGE_SIZE];
        self._seek(page_no)?;
        self.file.read(&mut buf)?;
        Ok(buf)
    }

    pub fn read_page(&mut self, page_no: P) -> Result<[u8; PAGE_SIZE]> {
        self._read(page_no.into())
    }

    pub fn read(&mut self, page_no: u64) -> Result<[u8; PAGE_SIZE]> {
        let mut buf = [0; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(PAGE_SIZE as u64 * page_no))?;
        self.file.read(&mut buf)?;
        Ok(buf)
    }

    pub fn write(&mut self, page_no: u64, buf: &[u8; PAGE_SIZE]) -> Result<usize> {
        self.file
            .seek(SeekFrom::Start(PAGE_SIZE as u64 * page_no))?;
        self.file.write(buf)
    }

    pub fn create(&mut self) -> Result<u64> {
        let len = self.file.seek(SeekFrom::End(0))?;
        self.file.set_len(len + PAGE_SIZE as u64)?; // todo: maybe remove this line?
        Ok(len / PAGE_SIZE as u64)
    }
}

impl<P, const PS: usize, const PAGE_SIZE: usize> Drop for Pages<P, PS, PAGE_SIZE> {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;

    use super::*;

    const FILE_PATH: &str = "data.idx";

    #[test]
    fn all() -> Result<()> {
        // create_new_page
        {
            let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 1);
            assert_eq!(pages.create()?, 2);
        }
        {
            let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 3);
        }

        // read_write_metadata
        {
            let msg = b"Hello, World";
            {
                let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
                let raw_meta = pages.metadata();
                raw_meta[0..msg.len()].copy_from_slice(msg);
                pages.sync_metadata()?;
            }
            {
                let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
                pages.clear()?; // It does't remove metadata
                let raw_meta = pages.metadata();
                assert_eq!(raw_meta[0..msg.len()], *msg);
            }
        }

        remove_file(FILE_PATH)?;
        Ok(())
    }

    #[test]
    #[ignore = "todo: refector, Invalid page size"]
    fn read_write_data() -> Result<()> {
        let mut pages: Pages<u16, 2, 12> = Pages::open("data2.idx")?;
        let page_on = pages.create()?;

        let msg = b"Hello, World";
        pages.write(page_on, msg)?;
        assert_eq!(pages.read(page_on)?, *msg);

        remove_file("data2.idx")?;
        Ok(())
    }
}
