pub mod endian;

pub struct BytesReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> BytesReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub fn next(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.cursor)?;
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
    pub fn buf<const S: usize>(&mut self) -> [u8; S] {
        self.get_buf().unwrap()
    }
    #[inline]
    pub fn bytes(&mut self, n: usize) -> &[u8] {
        self.get_bytes(n).unwrap()
    }
    #[inline]
    pub fn get_buf<const S: usize>(&mut self) -> Option<[u8; S]> {
        self.get_bytes(S)?.try_into().ok()
    }
    #[inline]
    pub fn get_bytes(&mut self, n: usize) -> Option<&[u8]> {
        Some(&self.bytes[self.cursor..self.advance(n)?])
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
                T::from_bytes(self.buf())
            }
        }
    )*);
}
impl_trait!(LittleEndian BigEndian NativeEndian);


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
