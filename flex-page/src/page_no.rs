use std::ops::{Add, Mul};

pub trait PageNo: Default /* + Mul + Add + TryFrom<u32>  */ {
    /// The size of this type in bytes
    const NBYTES: usize;
    fn new(_: usize) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Self;
}

macro_rules! impl_trait {
    ($name:ident for $($t:ty:$S:expr)*) => ($(
        impl $name for $t {
            const NBYTES: usize = $S;
            fn new(num: usize) -> Self { num.try_into().unwrap() }
            fn to_bytes(&self) -> Vec<u8> { self.to_le_bytes().to_vec() }
            fn from_bytes(bytes: Vec<u8>) -> Self { Self::from_le_bytes(bytes.try_into().unwrap()) }
        }
    )*)
}
impl_trait!(PageNo for u8:1 u16:2 u32:4);

#[derive(Default)]
pub struct U24(u32);

impl PageNo for U24 {
    const NBYTES: usize = 3;
    fn new(num: usize) -> Self {
        Self(num.try_into().unwrap())
    }
    fn to_bytes(&self) -> Vec<u8> {
        let [a, b, c, _] = self.0.to_le_bytes();
        vec![a, b, c]
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let buf: [u8; 3] = bytes.try_into().unwrap();
        let [a, b, c] = buf;
        Self(u32::from_le_bytes([a, b, c, 0]))
    }
}

// impl Mul for U24 {
//     type Output = Self;
//     fn mul(self, rhs: Self) -> Self {
//         (self.0 * rhs.0).into()
//     }
// }
// impl Add for U24 {
//     type Output = Self;
//     fn add(self, rhs: Self) -> Self {
//         (self.0 + rhs.0).into()
//     }
// }

// impl From<u32> for U24 {
//     fn from(num: u32) -> Self {
//         assert!(num < 16777215);
//         Self(num)
//     }
// }
