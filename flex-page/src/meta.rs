use crate::page_no::PageNo;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;

use data_view::DataView;

macro_rules! error { [$cond: expr, $msg: expr] => { if $cond { return Err(Error::new(ErrorKind::InvalidInput, $msg)); } }; }

pub struct Meta<K: PageNo, const NBYTES: usize> {
    size_info: SizeInfo,
    _marker:   PhantomData<K>,
}

impl<K: PageNo, const NBYTES: usize> Meta<K, NBYTES> {
    pub fn new() -> Result<Self> {
        error!(NBYTES < 64, "Page size should >= 64 bytes");
        error!(NBYTES > 1024 * 256, "Page size should > 256 kilobytes");
        let size_info = SizeInfo { block_size: NBYTES as u32, pages_len_nbytes: K::SIZE as u8 };
        Ok(Self { size_info, _marker: PhantomData })
    }

    pub fn to_bytes(&self) -> [u8; NBYTES] {
        let mut view = DataView::new([0; NBYTES]);
        view.write_slice(self.size_info.to_bytes());
        view.data
    }

    pub fn extend_from(&mut self, bytes: [u8; NBYTES]) -> Result<()> {
        let mut view = DataView::new(&bytes[..]);
        let size_info = SizeInfo::from(view.read_buf::<4>());
        error!(
            size_info != self.size_info,
            format!("Expected {:?}, but got: {:?}", self.size_info, size_info)
        );
        Ok(())
    }
}

/// Size Information, about the number of pages, and Page Size.
#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    /// Page Size
    block_size: u32,
    /// Binary size of Page Number. Possible values: (u8: 1, u16: 2, page_no::U24: 3, u32: 4)
    pages_len_nbytes: u8,
}

impl SizeInfo {
    fn to_bytes(&self) -> [u8; 4] {
        let [x, y, z, _] = self.block_size.to_le_bytes();
        [self.pages_len_nbytes, x, y, z]
    }

    fn from(bytes: [u8; 4]) -> Self {
        let [a, b, c, d] = bytes;
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
    fn metadata_binary_convertion() {
        let m1 = Meta::<u32, 2048>::new().unwrap();
        let mut m2 = Meta::<u32, 2048>::new().unwrap();
        m2.extend_from(m1.to_bytes()).unwrap();
        assert_eq!(m1.size_info, m2.size_info);
    }

    #[test]
    fn check_size_info() {
        let mut m1 = Meta::<u32, 4096>::new().unwrap();
        let m2 = Meta::<u16, 2048>::new().unwrap();

        let mut buf = [0; 4096];
        let m2_buf = m2.to_bytes();
        buf[..m2_buf.len()].copy_from_slice(&m2_buf);

        assert_eq!(
            "Expected SizeInfo { block_size: 4096, pages_len_nbytes: 4 }, but got: SizeInfo { block_size: 2048, pages_len_nbytes: 2 }",
            m1.extend_from(buf).err().unwrap().to_string()
        );
    }
}
