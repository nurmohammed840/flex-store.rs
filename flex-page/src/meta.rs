use crate::page_no::PageNo;
use data_view::DataView;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;

macro_rules! error { [$cond: expr, $msg: expr] => { if $cond { return Err(Error::new(ErrorKind::InvalidInput, $msg)); } }; }

pub struct Meta<K: PageNo, const NBYTES: usize> {
    size_info: SizeInfo,
    /// Last page num (pointer) of FreeList
    free_tail: u32,
    _marker:   PhantomData<K>,
    data:      &'static mut [u8],
}

impl<K, const NBYTES: usize> Meta<K, NBYTES>
where
    K: PageNo,
{
    pub(crate) fn new() -> Result<Self> {
        error!(NBYTES < 64, "Page size should >= 64 bytes");
        error!(NBYTES > 1024 * 512, "Page size should > 512 kilobytes");
        let size_info = SizeInfo { block_size: NBYTES as u32, pages_len_nbytes: K::SIZE as u8 };
        let data = Vec::with_capacity(NBYTES - 8).leak();
        Ok(Self { size_info, free_tail: 1, _marker: PhantomData, data })
    }

    pub(crate) fn to_bytes(&self) -> [u8; NBYTES] {
        let mut view = DataView::new([0; NBYTES]);
        view.write_slice(self.size_info.to_bytes());
        view.write::<u32>(self.free_tail);
        // view.write_slice(self.data);
        view.data
    }

    pub(crate) fn extend_from(&mut self, bytes: [u8; NBYTES]) -> Result<()> {
        let mut view = DataView::new(&bytes[..]);
        let size_info = SizeInfo::from(view.read_buf::<4>());
        error!(
            size_info != self.size_info,
            format!("Expected {:?}, but got: {:?}", self.size_info, size_info)
        );
        self.free_tail = view.read::<u32>();
        // self.data.copy_from_slice(view.remaining_slice());
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
        let mut m1 = Meta::<u32, 2048>::new().unwrap();
        let mut m2 = Meta::<u32, 2048>::new().unwrap();

        m1.free_tail = 42;
        m2.extend_from(m1.to_bytes()).unwrap();

        assert_eq!(m1.size_info, m2.size_info);
        assert_eq!(m1.free_tail, m2.free_tail);
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
