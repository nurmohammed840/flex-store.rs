macro_rules! impl_endians {
    [$name:ident $pre:ident $($rty:ty)*] => {
        pub trait $name<T> {
            fn get(&mut self) -> Option<T>;
            fn read(&mut self) -> T;
        }
        $(
            impl<'a> $name<$rty> for crate::BytesSeeker<'a> {
                #[inline]
                fn get(&mut self) -> Option<$rty> {
                    Some(<$rty>::$pre(self.get_buf()?))
                }
                #[inline]
                fn read(&mut self) -> $rty {
                    <$rty>::$pre(self.buf())
                }
            }
        )*
    };
    [$($name:ident: $pre:ident)*] => {
        $(impl_endians!($name $pre u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);)*
    }
}
impl_endians!(
    BytesReaderLE: from_le_bytes
    BytesReaderBE: from_be_bytes
    BytesReaderNE: from_ne_bytes
);