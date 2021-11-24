#[derive(Clone)]
pub struct ByteSeeker<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> ByteSeeker<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub fn next(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.cursor)?;
        self.cursor += 1;
        Some(byte)
    }

    pub fn get_buf<const S: usize>(&mut self) -> Option<[u8; S]> {
        self.get_octets(S)?.try_into().ok()
    }

    pub fn buf<const S: usize>(&mut self) -> [u8; S] {
        self.octets(S).try_into().unwrap()
    }

    pub fn get_octets(&mut self, n: usize) -> Option<&[u8]> {
        let len = self.cursor + n;
        if len > self.bytes.len() {
            return None;
        }
        let bytes = &self.bytes[self.cursor..len];
        self.cursor = len;
        Some(bytes)
    }

    pub fn octets(&mut self, n: usize) -> &[u8] {
        let len = self.cursor + n;
        let bytes = &self.bytes[self.cursor..len];
        self.cursor = len;
        bytes
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_streem() {
        let mut seeker = ByteSeeker::new(&[1u8, 2, 3, 4, 5, 6]);
        assert_eq!(Some(1), seeker.next());
        assert_eq!(Some(&[2, 3][..]), seeker.get_octets(2));
        assert_eq!(Some([5, 6]), seeker.get_buf());
    }
}
