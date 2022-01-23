use std::io::{Error, ErrorKind, Result};

macro_rules! err { [$cond: expr, $msg: expr] => { if $cond { return Err(Error::new(ErrorKind::InvalidInput, $msg)); } }; }

/// Size Information, about the number of pages, and Page Size.
#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    /// Page Size
    block_size: u32,
    /// Binary size of Page Number. Possible values: (u8: 1, u16: 2, page_no::U24: 3, u32: 4)
    pages_len_nbytes: u8,
}

impl SizeInfo {
    pub fn new(block_size: u32, pages_len_nbytes: u8) -> Result<Self> {
        err!(block_size < 64, "Page size should >= 64 bytes");
        err!(block_size > 1024 * 256, "Page size should > 256 kilobytes");
        Ok(Self { block_size, pages_len_nbytes })
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let [x, y, z, _] = self.block_size.to_le_bytes();
        [self.pages_len_nbytes, x, y, z]
    }

    pub fn check_from(& self, buf: [u8; 4]) -> Result<()> {
        let [a, b, c, d] = buf;
        let size_info = Self {
            pages_len_nbytes: a,
            block_size: u32::from_le_bytes([b, c, d, 0]),
        };
        err!(
            &size_info != self,
            format!("Expected {:?}, but got: {:?}", self, size_info)
        );
        Ok(())
    }
}