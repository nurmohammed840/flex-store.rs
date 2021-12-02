macro_rules! endians {
    [$($name:ident)*] => ($(
        pub trait $name<const S: usize> {
            fn to_bytes(&self) -> [u8; S];
            fn from_bytes(bytes: [u8; S]) -> Self;
        }
    )*);
    [$($t:ty : $nbytes:expr)*] => {$(
        impl LittleEndian<$nbytes> for $t {
            #[inline]
            fn to_bytes(&self) -> [u8; $nbytes] {
                self.to_le_bytes()
            }
            #[inline]
            fn from_bytes(bytes: [u8; $nbytes]) -> Self {
                Self::from_le_bytes(bytes)
            }
        }
        impl BigEndian<$nbytes> for $t {
            #[inline]
            fn to_bytes(&self) -> [u8; $nbytes] {
                self.to_be_bytes()
            }
            #[inline]
            fn from_bytes(bytes: [u8; $nbytes]) -> Self {
                Self::from_be_bytes(bytes)
            }
        }
        impl NativeEndian<$nbytes> for $t {
            #[inline]
            fn to_bytes(&self) -> [u8; $nbytes] {
                self.to_ne_bytes()
            }
            #[inline]
            fn from_bytes(bytes: [u8; $nbytes]) -> Self {
                Self::from_ne_bytes(bytes)
            }
        }
    )*};
}

endians!(LittleEndian BigEndian NativeEndian);
endians!(
    f32:4 f64:8
    u8:1 u16:2 u32:4 u64:8 u128:16
    i8:1 i16:2 i32:4 i64:8 i128:16
);
