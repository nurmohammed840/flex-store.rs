pub struct ByteSeeker(Vec<u8>, usize);

impl ByteSeeker {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes, 0)
    }
    
    /// will panic, If end of buffer
    pub fn first(&mut self) -> u8 {
        let v = self.0[self.1];
        self.1 += 1;
        v
    }
    
    /// will panic, If end of buffer
    pub fn get_buf<const S: usize>(&mut self) -> [u8; S] {
        let mut buf = [0; S];
        buf.copy_from_slice(&self.0[self.1..self.1 + S]);
        self.1 += S;
        buf
    }

    /// will panic, If end of buffer
    pub fn get_vec(&mut self, bytes_len: usize) -> Vec<u8> {
        let slice = self.0[self.1..self.1 + bytes_len].to_vec();
        self.1 += bytes_len;
        slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_streem() {
        let mut seeker = ByteSeeker::new(vec![1u8, 2, 3, 4, 5, 6]);
        assert_eq!(1, seeker.first());
        assert_eq!(vec![2, 3], seeker.get_vec(2));
        assert_eq!([4, 5, 6], seeker.get_buf::<3>());
    }
}
