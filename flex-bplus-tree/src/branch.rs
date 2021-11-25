use crate::entry::Key;

use byte_seeker::ByteSeeker;
use flex_page::PageNo;

pub struct Branch<K, P, const KS: usize, const PS: usize, const PAGE_SIZE: usize> {
    keys: Vec<K>,
    childs: Vec<P>,
}

impl<K, P, const KS: usize, const PS: usize, const PAGE_SIZE: usize> Branch<K, P, KS, PS, PAGE_SIZE>
where
    K: PartialOrd + Key<KS>,
    P: PageNo<PS>,
{
    fn max_keys_capacity() -> usize {
        // node_type + totel_len (keys.len + childs.len)
        let margin = 1 + 2;
        // Minus 1, Bcs keys_capacity is less by 1
        ((PAGE_SIZE - margin) / (KS + PS)) - 1
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PAGE_SIZE);

        let keys_len: u16 = self.keys.len().try_into().unwrap();
        let childs_len: u16 = self.childs.len().try_into().unwrap();
        bytes.extend((keys_len + childs_len).to_le_bytes());

        for key in &self.keys {
            bytes.extend_from_slice(&key.to_bytes());
        }
        for page_no in &self.childs {
            bytes.extend_from_slice(&page_no.to_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut seeker = ByteSeeker::new(bytes);
        let mut branch = Self::new();

        let keys_len = u16::from_le_bytes(seeker.buf()) / 2;

        // keys
        for _ in 0..keys_len {
            branch.keys.push(K::from_bytes(seeker.buf()));
        }
        // childs
        for _ in 0..(keys_len + 1) {
            branch.childs.push(P::from_bytes(seeker.buf()));
        }
        branch
    }

    pub fn new() -> Self {
        Self {
            keys: Vec::with_capacity(Self::max_keys_capacity() - 1),
            childs: Vec::with_capacity(Self::max_keys_capacity()),
        }
    }

    pub fn create_root(key: K, left: P, right: P) -> Self {
        let mut branch = Branch::new();
        branch.keys.push(key);
        branch.childs.push(left);
        branch.childs.push(right);
        branch
    }

    /// -> index
    pub fn lookup(&self, key: K) -> usize {
        let mut i = 0;
        for &_key in &self.keys {
            if key < _key {
                return i;
            }
            i += 1;
        }
        i
    }

    /// ### Panic: যদি `branch.childs`-এ কোনো উপাদান না থাকে, নিশ্চিত করুন যে `branch.childs`-এ অন্তত একটি উপাদান রয়েছে.
    pub fn update(&mut self, index: usize, (mid, page_no): (K, P)) {
        self.keys.insert(index, mid);
        self.childs.insert(index + 1, page_no);
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
