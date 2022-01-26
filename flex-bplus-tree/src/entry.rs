pub trait Key: Copy + PartialOrd +Send  {
    const SIZE: usize;
    fn to_bytes(self) -> [u8; Self::SIZE];
    fn from_bytes(bytes: [u8; Self::SIZE]) -> Self;
}
macro_rules! impl_key_for {
    [$($rty:ty : $nbyte:literal)*] => ($(
        impl Key for $rty {
            const SIZE: usize = $nbyte;
            #[inline(always)]
            fn to_bytes(self) -> [u8; Self::SIZE] { self.to_le_bytes() }
            #[inline(always)]
            fn from_bytes(bytes: [u8; Self::SIZE]) -> Self { Self::from_le_bytes(bytes) }
        }
    )*);
}
impl_key_for!(u8:1 u16:2 u32:4 u64:8 u128:16 i8:1 i16:2 i32:4 i64:8 i128:16 f32:4 f64:8);
impl<const N: usize> Key for [u8; N] {
    const SIZE: usize = N;
    fn to_bytes(self) -> [u8; Self::SIZE] { self[..].try_into().unwrap() }
    fn from_bytes(bytes: [u8; Self::SIZE]) -> Self { bytes[..].try_into().unwrap() }
}