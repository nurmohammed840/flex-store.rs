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
    fn max_keys_capacity() -> usize {
        // PAGE_SIZE -: 1 node type + 2 node len , 2 totel len (keys & childs) / 
        // total_size: X (key_size) + 2 (child_size) -
        // 1, Bcs keys_capacity is less by 1
        ((PAGE_SIZE - 5) / (X + 2)) - 1
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
            bytes.extend_from_slice(&p.to_le_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut seeker = ByteSeeker::new(bytes);
        let mut branch = Self::new();

        let keys_len = u16::from_le_bytes(seeker.buf()) / 2;

        // keys
        for _ in 0..keys_len {
            branch.keys.push(K::from_bytes(seeker.buf::<X>()));
        }
        // childs
        for _ in 0..(keys_len + 1) {
            branch.childs.push(u16::from_le_bytes(seeker.buf::<2>()));
        }
        branch
    }

    pub fn new() -> Self {
        Self {
            keys: Vec::with_capacity(Self::max_keys_capacity() - 1),
            childs: Vec::with_capacity(Self::max_keys_capacity()),
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
        let mut i = 0;
        for &_id in &self.keys {
            if id < _id {
                return i;
            }
            i += 1;
        }
        i
    }

    /// Panic: If there in no element in `childs`
    pub fn update(&mut self, i: usize, (mid, page_no): (K, u16)) {
        self.keys.insert(i, mid);
        self.childs.insert(i + 1, page_no);
    }

    pub fn is_full(&self) -> bool {
        self.keys.len() >= Self::max_keys_capacity()
    }

    pub fn split(&mut self) -> (Self, K) {
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
    type Branch = super::Branch<u64, 8, 4096>;

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
