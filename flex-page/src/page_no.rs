use std::ops::Mul;

pub trait PageNo: Default + Mul + Into<u32> {}

impl PageNo for u16 {}
impl PageNo for U24 {}
impl PageNo for u32 {}

#[derive(Default)]
pub struct U24(u32);
impl Mul for U24 {
    type Output = U24;
    fn mul(self, rhs: Self) -> Self::Output {
        U24(self.0 * rhs.0)
    }
}
impl Into<u32> for U24 {
    fn into(self) -> u32 {
        self.0
    }
}

// #[derive(Copy, Clone, Debug, Default)]
// #[repr(C)]
// pub struct u24(pub [u8; 3]);

// impl From<u32> for u24 {
//     fn from(num: u32) -> Self {
//         let [a, b, c, _] = num.to_le_bytes();
//         u24([a, b, c])
//     }
// }

// impl u24 {
//     pub fn as_num(self) -> u32 {
//         let u24([a, b, c]) = self;
//         u32::from_le_bytes([a, b, c, 0])
//     }
// }
