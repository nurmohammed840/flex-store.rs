pub trait PageNo {
    fn to_page_no(&self) -> u64;
}

impl PageNo for u16 {
    fn to_page_no(&self) -> u64 {
        *self as u64
    }
}

impl PageNo for u24 {
    fn to_page_no(&self) -> u64 {
        self.as_num() as u64
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct u24(pub [u8; 3]);

impl From<u32> for u24 {
    fn from(num: u32) -> Self {
        let [a, b, c, _] = num.to_le_bytes();
        u24([a, b, c])
    }
}

impl u24 {
    pub fn as_num(self) -> u32 {
        let u24([a, b, c]) = self;
        u32::from_le_bytes([a, b, c, 0])
    }
}
