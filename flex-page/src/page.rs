struct Page<const S: usize> {
    no: u64,
    buf: [u8; S],
}

impl<const S: usize> Page<S> {
    fn new(no: u64, buf: [u8; S]) -> Self {
        Self { no, buf }
    }
    
}
