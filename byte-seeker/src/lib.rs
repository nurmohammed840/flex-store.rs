pub mod reader;
pub mod writer;

pub use reader::*;
pub use writer::*;

pub struct BytesSeeker<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> BytesSeeker<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    #[inline]
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
        self.cursor = len;
        Some(len)
    }

    #[inline]
    pub fn get_bytes(&mut self, n: usize) -> Option<&[u8]> {
        Some(&self.bytes[self.cursor..self.advance(n)?])
    }

    #[inline]
    pub fn get_buf<const S: usize>(&mut self) -> Option<[u8; S]> {
        self.get_bytes(S)?.try_into().ok()
    }

    #[inline]
    pub fn bytes(&mut self, n: usize) -> &[u8] {
        let len = self.cursor + n;
        let bytes = &self.bytes[self.cursor..len];
        self.cursor = len;
        bytes
    }
    #[inline]
    pub fn buf<const S: usize>(&mut self) -> [u8; S] {
        self.bytes(S).try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::BytesSeeker;

    #[test]
    fn advance() {
        let mut reader = BytesSeeker::new(&[1, 2]);
        assert_eq!(None, reader.advance(100));
        assert_eq!(Some(1), reader.advance(1));
        assert_eq!(None, reader.advance(2));
        assert_eq!(Some(2), reader.advance(1));
        assert_eq!(None, reader.advance(1));
    }

    #[test]
    fn get_bytes() {
        let mut reader = BytesSeeker::new(&[1, 2]);
        assert_eq!(None, reader.get_bytes(100));
        assert_eq!(Some(&[1][..]), reader.get_bytes(1));
        assert_eq!(None, reader.get_bytes(2));
        assert_eq!(Some(&[2][..]), reader.get_bytes(1));
        assert_eq!(None, reader.get_bytes(1));
    }
}
