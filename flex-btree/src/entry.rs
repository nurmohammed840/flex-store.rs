use std::{convert::TryInto, fmt::Debug};

pub trait Key: Copy + PartialOrd + Send + Sync + Unpin + Debug {
	const SIZE: usize;
	fn to_bytes(self) -> Vec<u8>;
	fn from_bytes(bytes: &[u8]) -> Self;
}
macro_rules! impl_key_for {
    [$($rty:ty : $nbyte:literal)*] => ($(
        impl Key for $rty {
            const SIZE: usize = $nbyte;
            #[inline]
            fn to_bytes(self) -> Vec<u8> { self.to_le_bytes().to_vec() }
            #[inline]
            fn from_bytes(bytes: &[u8]) -> Self { Self::from_le_bytes(bytes.try_into().unwrap()) }
        }
    )*);
}
impl_key_for!(u8:1 u16:2 u32:4 u64:8 u128:16 i8:1 i16:2 i32:4 i64:8 i128:16 f32:4 f64:8);

impl<const N: usize> Key for [u8; N] {
	const SIZE: usize = N;
	fn to_bytes(self) -> Vec<u8> {
		self.to_vec()
	}
	fn from_bytes(bytes: &[u8]) -> Self {
		bytes.try_into().unwrap()
	}
}
