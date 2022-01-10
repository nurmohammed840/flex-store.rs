use std::hash::Hash;

pub trait PageNo: Eq + Hash + Unpin + Copy {
    /// Total number of bytes.
    const SIZE: usize;
    fn new(_: u32) -> Self;
    fn as_u32(self) -> u32;
    fn to_bytes(&self) -> [u8; Self::SIZE];
    fn from_bytes(_: [u8; Self::SIZE]) -> Self;
}
macro_rules! impl_trait {
    ($name:ident for $($t:ty:$S:expr)*) => ($(
        impl $name for $t {
            const SIZE: usize = $S;
            #[inline]
            fn new(num: u32) -> Self { num.try_into().unwrap() }
            #[inline]
            fn as_u32(self) -> u32 { self.try_into().unwrap() }
            #[inline]
            fn to_bytes(&self) -> [u8; Self::SIZE] { self.to_le_bytes() }
            #[inline]
            fn from_bytes(buf: [u8; Self::SIZE]) -> Self { Self::from_le_bytes(buf) }
        }
    )*)
}
impl_trait!(PageNo for u8:1 u16:2 u32:4);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct U24(u32);

impl PageNo for U24 {
    const SIZE: usize = 3;
    fn new(num: u32) -> Self {
        assert!(num < 16777215);
        Self(num)
    }
    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let [a, b, c, _] = self.0.to_le_bytes();
        [a, b, c]
    }
    fn from_bytes(buf: [u8; Self::SIZE]) -> Self {
        let [a, b, c] = buf;
        Self(u32::from_le_bytes([a, b, c, 0]))
    }
    fn as_u32(self) -> u32 {
        self.0
    }
}
