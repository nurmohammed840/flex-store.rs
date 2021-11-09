#[derive(Clone)]
pub struct ByteSeeker<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Iterator for ByteSeeker<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.cursor)?;
        self.cursor += 1;
        Some(byte)
    }
}

impl<'a> ByteSeeker<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        let len = self.cursor + n;
        let bytes_len = self.bytes.len();
        if len > bytes_len {
            let current_len = self.cursor;
            self.cursor = bytes_len;
            return Err(bytes_len - current_len);
        }
        self.cursor = len;
        Ok(())
    }

    pub fn buf<const S: usize>(&mut self) -> Option<[u8; S]> {
        let len = self.cursor + S;
        if len > self.bytes.len() {
            return None;
        }
        let mut buf = [0; S];
        buf.copy_from_slice(&self.bytes[self.cursor..len]);
        self.cursor += S;
        Some(buf)
    }

    pub fn octets(&mut self, n: usize) -> Option<Vec<u8>> {
        let len = self.cursor + n;
        if len > self.bytes.len() {
            return None;
        }
        let bytes = self.bytes[self.cursor..len].to_vec();
        self.cursor += n;
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_streem() {
        let mut seeker = ByteSeeker::new(&[1u8, 2, 3, 4, 5, 6]);
        assert_eq!(Some(1), seeker.next());
        assert_eq!(Some(vec![2, 3]), seeker.octets(2));
        assert_eq!(Ok(()), seeker.advance_by(1));
        assert_eq!(Some([5, 6]), seeker.buf::<2>());
        assert_eq!(Err(0), seeker.advance_by(100));
    }
}
