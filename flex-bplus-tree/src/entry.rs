use std::{fmt::Debug, mem::size_of};

pub trait Key<const S: usize>: Copy {
    fn to_bytes(&self) -> [u8; S];
    fn from_bytes(bytes: [u8; S]) -> Self;
}

#[derive(Debug)]
pub struct Entry<K, V, const X: usize, const Y: usize> {
    pub key: K,
    pub value: V,
}

impl<K: Key<X>, V: Key<Y>, const X: usize, const Y: usize> Entry<K, V, X, Y> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(size_of::<Self>());
        bytes.extend(self.key.to_bytes());
        bytes.extend(self.value.to_bytes());
        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let l = size_of::<K>();
        Self {
            key: K::from_bytes(bytes[..l].try_into().unwrap()),
            value: V::from_bytes(bytes[l..].try_into().unwrap()),
        }
    }
}

macro_rules! impl_trait {
    ($name:ident for $($S:expr;$t:ty)*) => ($(
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
impl_trait!(Key for 4;f32 8;f64);
impl_trait!(Key for 1;u8 2;u16 4;u32 8;u64 16;u128);
impl_trait!(Key for 1;i8 2;i16 4;i32 8;i64 16;i128);

impl<const S: usize> Key<S> for [u8; S] {
    fn to_bytes(&self) -> [u8; S] {
        *self
    }
    fn from_bytes(bytes: [u8; S]) -> Self {
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_bytes() {}
    #[test]
    fn from_bytes() {}
}