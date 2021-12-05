pub mod endian;

pub struct BytesReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> BytesReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    #[inline]
    pub fn next(&mut self) -> Option<&u8> {
        let byte = self.bytes.get(self.cursor)?;
        self.cursor += 1;
        Some(byte)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) -> Option<usize> {
        let len = self.cursor + n;
        if len > self.bytes.len() {
            return None;
        }
        self.cursor += n;
        Some(self.cursor)
    }

    #[inline]
    pub fn get_bytes(&mut self, n: usize) -> Option<&[u8]> {
        Some(&self.bytes[self.cursor..self.advance(n)?])
    }

    #[inline]
    pub fn read_bytes(&mut self, n: usize) -> &[u8] {
        self.get_bytes(n).unwrap()
    }

    #[inline]
    pub fn get_buf<const S: usize>(&mut self) -> Option<[u8; S]> {
        self.get_bytes(S)?.try_into().ok()
    }

    #[inline]
    pub fn read_buf<const S: usize>(&mut self) -> [u8; S] {
        self.get_buf().unwrap()
    }
}

macro_rules! impl_trait {
    [$($name:ident)*] => ($(
        pub trait $name {
            fn get<T: endian::$name<S>, const S: usize>(&mut self) -> Option<T>;
            fn read<T: endian::$name<S>, const S: usize>(&mut self) -> T;
        }
        impl<'a> $name for BytesReader<'a> {
            #[inline]
            fn get<T: endian::$name<S>, const S: usize>(&mut self) -> Option<T> {
                Some(T::from_bytes(self.get_buf()?))
            }
            #[inline]
            fn read<T: endian::$name<S>, const S: usize>(&mut self) -> T {
                T::from_bytes(self.read_buf())
            }
        }
    )*);
}
impl_trait!(LittleEndian BigEndian NativeEndian);

#[cfg(test)]
mod testas {
    #[test]
    fn test_name() {}
}

#[cfg(test)]
mod tests {
    use super::{BytesReader, LittleEndian};
    #[test]
    fn byte_streem() {
        let mut seeker = BytesReader::new(&[1u8, 2, 3, 4, 5, 6]);
        assert_eq!(Some(1u8), seeker.get());
        assert_eq!(Some(&[2, 3][..]), seeker.get_bytes(2));
        assert_eq!(Some([4, 5]), seeker.get_buf());
        assert_eq!(6u8, seeker.read());
        assert_eq!(None, seeker.get_bytes(10));
    }
}

macro_rules! aaa {
    [$name:ident $pre:ident $($m1:ident, $m2:ident -> $ret_ty:ty)*] => {
        pub trait $name {
            $(
                fn $m1(&mut self) -> $ret_ty;
                fn $m2(&mut self) -> Option<$ret_ty>;
            )*
        }
        impl<'a> $name for BytesReader<'a> {
            $(
                #[inline]
                fn $m1(&mut self) -> $ret_ty {
                    <$ret_ty>::from_be_bytes(self.read_buf())
                }
                #[inline]
                fn $m2(&mut self) -> Option<$ret_ty> {
                    Some(<$ret_ty>::from_be_bytes(self.get_buf()?))
                }
            )*
        }
    };
    [$($name:ident: $pre:ident)*] => {$(
        aaa!($name $pre
            read_u8, get_u8 -> u8
            read_u16, get_u16 -> u16
            read_u32, get_u32 -> u32
            read_u64, get_u64 -> u64
            read_u128, get_u128 -> u128
            read_i8, get_i8 -> i8
            read_i16, get_i16 -> i16
            read_i32, get_i32 -> i32
            read_i64, get_i64 -> i64
            read_i128, get_i128 -> i128
            read_f32, get_f32 -> f32
            read_f64, get_f64 -> f64
        );
    )*}
}
aaa!(
    LE: from_le_bytes
    BE: from_be_bytes
    NE: from_ne_bytes
);


