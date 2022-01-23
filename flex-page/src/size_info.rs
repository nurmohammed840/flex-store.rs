/// Size Information, about the number of pages, and Page Size.
#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    /// Page Size
    pub block_size: u32,
    /// Binary size of Page Number. Possible values: (u8: 1, u16: 2, page_no::U24: 3, u32: 4)
    pub pages_len_nbytes: u8,
}

impl SizeInfo {
    pub fn to_bytes(&self) -> [u8; 4] {
        let [x, y, z, _] = self.block_size.to_le_bytes();
        [self.pages_len_nbytes, x, y, z]
    }

    pub fn from(buf: [u8; 4]) -> Self {
        let [a, b, c, d] = buf;
        Self { pages_len_nbytes: a, block_size: u32::from_le_bytes([b, c, d, 0]) }
    }
}
