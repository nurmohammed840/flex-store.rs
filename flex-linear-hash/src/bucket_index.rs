use flex_page::PageNo;

#[repr(C)]
pub struct BucketIndex {
    pub round: u8,
    pub pointer: PageNo::u24,
}

impl BucketIndex {
    pub fn to_bytes(self) -> [u8; 4] {
        unsafe { std::mem::transmute::<Self, [u8; 4]>(self) }
    }
    pub fn from_bytes(buf: [u8; 4]) -> Self {
        unsafe { std::mem::transmute::<[u8; 4], Self>(buf) }
    }
}
