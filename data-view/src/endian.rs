pub trait Endian {
    const NBYTES: usize;
    fn to_bytes_le(self) -> [u8; Self::NBYTES];
    fn to_bytes_be(self) -> [u8; Self::NBYTES];
    fn to_bytes_ne(self) -> [u8; Self::NBYTES];
    fn from_bytes_le(bytes: [u8; Self::NBYTES]) -> Self;
    fn from_bytes_be(bytes: [u8; Self::NBYTES]) -> Self;
    fn from_bytes_ne(bytes: [u8; Self::NBYTES]) -> Self;
}
macro_rules! impl_endian_ext {
    [$($rty:ty : $nbyte:literal)*] => ($(
        impl Endian for $rty {
            const NBYTES: usize = $nbyte;
            #[inline(always)]
            fn to_bytes_le(self) -> [u8; Self::NBYTES] { self.to_le_bytes() }
            #[inline(always)]
            fn to_bytes_be(self) -> [u8; Self::NBYTES] { self.to_be_bytes() }
            #[inline(always)]
            fn to_bytes_ne(self) -> [u8; Self::NBYTES] { self.to_ne_bytes() }
            #[inline(always)]
            fn from_bytes_le(bytes: [u8; Self::NBYTES]) -> Self { Self::from_le_bytes(bytes) }
            #[inline(always)]
            fn from_bytes_be(bytes: [u8; Self::NBYTES]) -> Self { Self::from_be_bytes(bytes) }
            #[inline(always)]
            fn from_bytes_ne(bytes: [u8; Self::NBYTES]) -> Self { Self::from_ne_bytes(bytes) }
        }
    )*);
}
impl_endian_ext!(u8:1 u16:2 u32:4 u64:8 u128:16 i8:1 i16:2 i32:4 i64:8 i128:16 f32:4 f64:8);
