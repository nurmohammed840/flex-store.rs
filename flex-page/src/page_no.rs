use std::ops::Mul;

pub trait PageNo<const S: usize>: Default + Mul + Into<u32> {
    const SIZE: usize = S;
    fn to_bytes(&self) -> [u8; S];
    fn from_bytes(bytes: [u8; S]) -> Self;
}

#[derive(Default)]
pub struct U24(u32);

impl From<u32> for U24 {
    fn from(num: u32) -> Self {
        assert!(num < 16777215);
        Self(num)
    }
}

impl Mul for U24 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        (self.0 * rhs.0).into()
    }
}

impl Into<u32> for U24 {
    fn into(self) -> u32 {
        self.0
    }
}

macro_rules! impl_trait {
    ($name:ident for $($t:ty:$S:expr)*) => ($(
        impl $name<$S> for $t {
            #[inline]
            fn to_bytes(&self) -> [u8; $S] {
                self.to_le_bytes()
            }
            #[inline]
            fn from_bytes(bytes: [u8; $S]) -> Self {
                Self::from_le_bytes(bytes)
            }
        }
    )*)
}

impl_trait!(PageNo for u16:2 u32:4);

impl PageNo<3> for U24 {
    fn to_bytes(&self) -> [u8; 3] {
        let [a, b, c, _] = self.0.to_le_bytes();
        [a, b, c]
    }
    fn from_bytes(bytes: [u8; 3]) -> Self {
        let [a, b, c] = bytes;
        Self(u32::from_le_bytes([a, b, c, 0]))
    }
}
