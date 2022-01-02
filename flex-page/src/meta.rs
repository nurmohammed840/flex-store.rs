use std::{
    io::{Cursor, Error, ErrorKind, Result, Write},
    sync::atomic::{AtomicU32, Ordering},
};

use crate::page_no::PageNo;
use utils::cursor::Reader;

pub struct Meta<K: PageNo, const NBYTES: usize>
where
    [u8; NBYTES - 8]:,
{
    size_info: SizeInfo,
    /// Free list page pointer
    last_free_page: AtomicU32,
    pub data: [u8; NBYTES - 8],
}

impl<K, const NBYTES: usize> Meta<K, NBYTES>
where
    K: PageNo,
    [u8; NBYTES - 8]:,
{
    pub fn new() -> Self {
        assert!(NBYTES >= 64, "Page size should >= 64 bytes");
        assert!(NBYTES < 16777216, "Page size should < 16mb");
        Self {
            size_info: SizeInfo {
                block_size: NBYTES as u32,
                pages_len_nbytes: K::SIZE as u8,
            },
            last_free_page: AtomicU32::new(1),
            data: [0; NBYTES - 8],
        }
    }

    pub fn last_free_page(&self) -> u32 {
        self.last_free_page.load(Ordering::SeqCst)
    }

    /// This funtion return expected `SizeInfo` as error.
    pub(crate) fn extend_from(&mut self, bytes: [u8; NBYTES]) -> Result<()>
    where
        [u8; K::SIZE]:,
    {
        let mut reader = Cursor::new(&bytes);
        let size_info = SizeInfo::from(reader.buf::<4>()?);

        if size_info != self.size_info {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Expected {:?}, but got: {:?}", self.size_info, size_info),
            ));
        }
        self.last_free_page = AtomicU32::new(reader.get::<u32>()?);
        self.data.copy_from_slice(reader.remaining_slice());
        Ok(())
    }

    pub(crate) fn to_bytes(&self) -> [u8; NBYTES]
    where
        [u8; K::SIZE]:,
    {
        let mut buf = [0; NBYTES];
        let mut bytes = Cursor::new(buf.as_mut());
        bytes.write(&self.size_info.to_bytes()).unwrap();
        bytes
            .write(&self.last_free_page.load(Ordering::Relaxed).to_le_bytes())
            .unwrap();
        bytes.write(&self.data).unwrap();
        buf
    }
}

#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    block_size: u32,
    pages_len_nbytes: u8,
}
impl SizeInfo {
    fn to_bytes(&self) -> [u8; 4] {
        let [x, y, z, _] = self.block_size.to_le_bytes();
        [self.pages_len_nbytes, x, y, z]
    }
    fn from(buf: [u8; 4]) -> Self {
        let [a, b, c, d] = buf;
        Self {
            pages_len_nbytes: a,
            block_size: u32::from_le_bytes([b, c, d, 0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_size_info() {
        let m1 = Meta::<u16, 4096>::new();
        let mut m2 = Meta::<u32, 8192>::new();
        assert_eq!(m1.to_bytes().len(), 8);
        assert_eq!(m2.to_bytes().len(), 10);
        // assert_eq!(
        //     "Expected SizeInfo { page_size: 8192, page_len_nbytes: 4 }, but got: SizeInfo { page_size: 4096, page_len_nbytes: 2 }",
        //      m2.extend_from(m1.to_bytes()).err().unwrap().to_string()
        // );
    }
}
