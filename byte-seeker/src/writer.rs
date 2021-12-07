macro_rules! impls_writers {
    [$name:ident $pre:ident $($nty:ty)*] => {
        pub trait $name<T> {
            fn write(&mut self, _: T);
        }
        $(
            impl $name<$nty> for Vec<u8> {
                #[inline]
                fn write(&mut self, num: $nty) {
                    self.extend(<$nty>::$pre(num))
                }
            }
        )*
    };
    [$($name:ident: $pre:ident)*]=> {$(
        impls_writers!($name $pre u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);
    )*}
}

impls_writers!(
    BytesWriterLE: to_le_bytes
    BytesWriterBE: to_be_bytes
    BytesWriterNE: to_ne_bytes
);
