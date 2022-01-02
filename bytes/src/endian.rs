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
            #[inline]
            fn to_bytes_le(self) -> [u8; Self::NBYTES] { self.to_le_bytes() }
            #[inline]
            fn to_bytes_be(self) -> [u8; Self::NBYTES] { self.to_be_bytes() }
            #[inline]
            fn to_bytes_ne(self) -> [u8; Self::NBYTES] { self.to_ne_bytes() }
            #[inline]
            fn from_bytes_le(bytes: [u8; Self::NBYTES]) -> Self { Self::from_le_bytes(bytes) }
            #[inline]
            fn from_bytes_be(bytes: [u8; Self::NBYTES]) -> Self { Self::from_be_bytes(bytes) }
            #[inline]
            fn from_bytes_ne(bytes: [u8; Self::NBYTES]) -> Self { Self::from_ne_bytes(bytes) }
        }
    )*);
}
impl_endian_ext!(u8:1 u16:2 u32:4 u64:8 u128:16 i8:1 i16:2 i32:4 i64:8 i128:16 f32:4 f64:8);

pub trait DataView {
    fn get<T>(&self, _: usize) -> T
    where
        T: Endian,
        [u8; T::NBYTES]:;

    fn set<T>(&mut self, _: usize, _: T)
    where
        T: Endian,
        [u8; T::NBYTES]:;
    
    fn set_bytes(&mut self, _: usize, _: &[u8]);
}
macro_rules! DefaultView {
    [] => (
        #[inline]
        fn get<T>(&self, offset: usize) -> T
        where
            T: Endian,
            [u8; T::NBYTES]:,
        {
            let bytes = self[offset..(offset + T::NBYTES)].try_into().unwrap();
            #[cfg(not(any(feature = "big", feature = "native")))]
            return T::from_bytes_le(bytes);
            #[cfg(feature = "big")]
            return T::from_bytes_be(bytes);
            #[cfg(feature = "native")]
            return T::from_bytes_ne(bytes);
        }
        #[inline]
        fn set<T>(&mut self, offset: usize, value: T)
        where
            T: Endian,
            [u8; T::NBYTES]:,
        {
            #[cfg(not(any(feature = "big", feature = "native")))]
            self.set_bytes(offset, &T::to_bytes_le(value));
            #[cfg(feature = "big")]
            self.set_bytes(offset, &T::to_bytes_be(value));
            #[cfg(feature = "native")]
            self.set_bytes(offset, &T::to_bytes_ne(value));
        }
        #[inline]
        fn set_bytes(&mut self, offset: usize, bytes: &[u8]) {
            self[offset..(offset + bytes.len())].copy_from_slice(bytes);
        }
    );
}

impl DataView for Vec<u8> {
    DefaultView!();
}
impl<const S: usize> DataView for [u8; S] {
    DefaultView!();
}