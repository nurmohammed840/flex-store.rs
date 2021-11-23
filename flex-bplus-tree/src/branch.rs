use byte_seeker::ByteSeeker;

pub struct Branch<Key, PageNo, const KEY: usize, const PAGE_NO: usize, const PAGE_SIZE: usize> {
    pub keys: Vec<Key>,
    pub childs: Vec<PageNo>,
}

impl<Key, PageNo, const KEY: usize, const PAGE_NO: usize, const PAGE_SIZE: usize>
    Branch<Key, PageNo, KEY, PAGE_NO, PAGE_SIZE>
where
    Key: PartialOrd + crate::entry::Key<KEY>,
    PageNo: flex_page::PageNo<PAGE_NO>,
{
    fn max_keys_capacity() -> usize {
        // node type + node len + totel len (keys & childs)
        let margin = 1 + 2 + 2;
        // Minus 1, Bcs keys_capacity is less by 1
        ((PAGE_SIZE - margin) / (KEY + PAGE_NO)) - 1
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PAGE_SIZE);

        let keys_len: u16 = self.keys.len().try_into().unwrap();
        let childs_len: u16 = self.childs.len().try_into().unwrap();
        bytes.extend((keys_len + childs_len).to_le_bytes());

        for k in &self.keys {
            bytes.extend_from_slice(&k.to_bytes());
        }
        for p in &self.childs {
            bytes.extend_from_slice(&p.to_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut seeker = ByteSeeker::new(bytes);
        let mut branch = Self::new();

        let keys_len = u16::from_le_bytes(seeker.buf()) / 2;

        // keys
        for _ in 0..keys_len {
            branch.keys.push(Key::from_bytes(seeker.buf()));
        }
        // childs
        for _ in 0..(keys_len + 1) {
            branch.childs.push(PageNo::from_bytes(seeker.buf()));
        }
        branch
    }

    pub fn new() -> Self {
        Self {
            keys: Vec::with_capacity(Self::max_keys_capacity() - 1),
            childs: Vec::with_capacity(Self::max_keys_capacity()),
        }
    }

    pub fn create_root(key: Key, left: PageNo, right: PageNo) -> Self {
        let mut branch = Branch::new();
        branch.keys.push(key);
        branch.childs.push(left);
        branch.childs.push(right);
        branch
    }

    /// -> index
    pub fn lookup(&self, key: Key) -> usize {
        let mut i = 0;
        for &_key in &self.keys {
            if key < _key {
                return i;
            }
            i += 1;
        }
        i
    }

    /// Panic: If there in no element in `childs`
    pub fn update(&mut self, i: usize, (mid, page_no): (Key, PageNo)) {
        self.keys.insert(i, mid);
        self.childs.insert(i + 1, page_no);
    }

    pub fn is_full(&self) -> bool {
        self.keys.len() >= Self::max_keys_capacity()
    }

    pub fn split(&mut self) -> (Self, Key) {
        let mid_point = Self::max_keys_capacity() / 2;
        let right = Self {
            keys: self.keys.drain(mid_point..).collect(),
            childs: self.childs.drain(mid_point..).collect(),
        };
        (right, self.keys.pop().unwrap())
    }
}

#[cfg(test)]
mod tests {
    type Branch = super::Branch<u64, u16, 8, 2, 4096>;

    #[test]
    fn lookup() {
        let mut branch: Branch = Branch::new();
        branch.keys = vec![10u64, 15, 20];

        assert_eq!(branch.lookup(10), 1);
        assert_eq!(branch.lookup(15), 2);
        assert_eq!(branch.lookup(20), 3);

        assert_eq!(branch.lookup(0), 0);
        assert_eq!(branch.lookup(9), 0);
        assert_eq!(branch.lookup(11), 1);
        assert_eq!(branch.lookup(14), 1);
        assert_eq!(branch.lookup(16), 2);
        assert_eq!(branch.lookup(19), 2);
        assert_eq!(branch.lookup(21), 3);
        assert_eq!(branch.lookup(100), 3);
    }

    #[test]
    fn split() {
        let mut left: Branch = Branch::create_root(0, 0, 1);

        for i in 1..Branch::max_keys_capacity() {
            left.update(i, (i as u64, i as u16 + 1));
        }

        assert!(left.is_full());

        // ------------------  to/from byte test ------------------
        let bytes = left.to_bytes();
        assert_eq!(bytes.len(), 4084);
        assert_eq!(Branch::from_bytes(&bytes).to_bytes(), bytes);
        // --------------------------------------------------------

        let (right, mid) = left.split();

        assert_eq!(mid, 203);

        assert_eq!(left.keys, (0..=202).collect::<Vec<u64>>());
        assert_eq!(right.keys, (204..=407).collect::<Vec<u64>>());

        assert_eq!(left.childs, (0..=203).collect::<Vec<u16>>());
        assert_eq!(right.childs, (204..=408).collect::<Vec<u16>>());
    }
}
