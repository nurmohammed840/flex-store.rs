use std::mem;

use crate::entry::Key;
use byte_seeker::ByteSeeker;

pub struct Branch<K, const X: usize, const PAGE_SIZE: usize> {
    pub keys: Vec<K>,
    pub childs: Vec<u16>,
}

impl<K, const X: usize, const PAGE_SIZE: usize> Branch<K, X, PAGE_SIZE>
where
    K: PartialOrd + Key<X>,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((Self::max_childs_capacity() * 2) - 1);

        let keys_len: u16 = self.keys.len().try_into().unwrap();
        let childs_len: u16 = self.childs.len().try_into().unwrap();
        let len = keys_len + childs_len;
        bytes.extend(len.to_le_bytes());

        for k in &self.keys {
            bytes.extend_from_slice(&k.to_bytes());
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut seeker = ByteSeeker::new(bytes);
        let mut branch = Self::new();

        let len = u16::from_le_bytes(seeker.buf_unwrap());
        let keys_len = len / 2;
        
        // keys
        for _ in 0..keys_len {
            branch.keys.push(K::from_bytes(seeker.buf_unwrap::<X>()));
        }
        // childs
        for _ in 0..(keys_len + 1) {
            branch.childs.push(u16::from_le_bytes(seeker.buf_unwrap::<2>()));
        }
        branch
    }

    fn max_childs_capacity() -> usize {
        // 1 byte for node type, 2 bytes for len = 3
        let page_size = PAGE_SIZE - 3;
        let mut n = page_size / (mem::size_of::<K>() + 2);
        // 2 bytes for extra `childs` capacity
        if page_size % n >= 2 {
            n += 1;
        }
        n
    }

    pub fn new() -> Self {
        Self {
            keys: Vec::with_capacity(Self::max_childs_capacity() - 1),
            childs: Vec::with_capacity(Self::max_childs_capacity()),
        }
    }

    pub fn create_root(id: K, left_child: u16, right_child: u16) -> Self {
        let mut branch = Branch::new();
        branch.keys.push(id);
        branch.childs.push(left_child);
        branch.childs.push(right_child);
        branch
    }

    /// -> index
    pub fn lookup(&self, id: K) -> usize {
        let mut i: usize = 0;
        for _id in self.keys.iter() {
            if id < *_id {
                return i;
            }
            i += 1;
        }
        i
    }

    pub fn update(&mut self, i: usize, (mid, page_no): (K, u16)) {
        self.keys.insert(i, mid);
        self.childs.insert(i + 1, page_no);
    }

    pub fn is_full(&self) -> bool {
        self.childs.len() >= Self::max_childs_capacity()
    }

    pub fn split(&mut self) -> (Self, K) {
        let mut right = Self::new();

        let mid_point = Self::max_childs_capacity() / 2;

        right.keys = self.keys.drain(mid_point..).collect();
        right.childs = self.childs.drain(mid_point..).collect();

        (right, self.keys.pop().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type B = Branch<u64, 8, 4096>;

    #[test]
    fn lookup() {
        let mut branch: B = Branch::new();
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
        let mut left: B = Branch::create_root(0, 0, 1);

        for i in 1..B::max_childs_capacity() {
            left.update(i, (i as u64, i as u16 + 1));
        }

        assert!(left.is_full());

        let (right, mid) = left.split();

        assert_eq!(mid, 204);

        assert_eq!(left.keys, (0..=203).collect::<Vec<u64>>());
        assert_eq!(right.keys, (205..=409).collect::<Vec<u64>>());

        assert_eq!(left.childs, (0..=204).collect::<Vec<u16>>());
        assert_eq!(right.childs, (205..=410).collect::<Vec<u16>>());
    }
}
