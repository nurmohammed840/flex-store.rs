use crate::entry::{Entry, Key};
use byte_seeker::ByteSeeker;
use flex_page::PageNo;

pub enum SetOption {
    UpdateOrInsert,
    FindOrInsert,
}

#[derive(Debug)]
pub struct Leaf<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
{
    left: P,
    right: P,
    entrys: Vec<Entry<K, V, KS, VS>>,
}

impl<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
    Leaf<K, V, P, KS, VS, PS, PAGE_SIZE>
where
    K: Ord + Key<KS>,
    V: Key<VS>,
    P: PageNo<PS>,
{
    pub fn max_entrys_capacity() -> usize {
        //  node_type + (left + right) + entrys.len
        let margin = 1 + (PS + PS) + 2;
        (PAGE_SIZE - margin) / (KS + VS)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PAGE_SIZE);

        bytes.extend_from_slice(&self.left.to_bytes());
        bytes.extend_from_slice(&self.right.to_bytes());

        let len: u16 = self.entrys.len().try_into().unwrap();
        bytes.extend_from_slice(&len.to_le_bytes());

        for entry in &self.entrys {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut byte_seeker = ByteSeeker::new(bytes);
        let mut leaf = Self::new();

        leaf.left = P::from_bytes(byte_seeker.buf());
        leaf.right = P::from_bytes(byte_seeker.buf());

        let len = u16::from_le_bytes(byte_seeker.buf());

        for _ in 0..len {
            let bytes = byte_seeker.octets(KS + VS);
            leaf.entrys.push(Entry::<K, V, KS, VS>::from_bytes(&bytes));
        }
        leaf
    }

    pub fn new() -> Self {
        Self {
            entrys: Vec::with_capacity(Self::max_entrys_capacity()),
            left: P::default(),
            right: P::default(),
        }
    }

    fn binary_search_by(&self, key: &K) -> Result<usize, usize> {
        self.entrys.binary_search_by_key(&key, |entry| &entry.key)
    }

    /// Insert and sort `entrys`
    pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> V {
        match opt {
            SetOption::FindOrInsert => match self.binary_search_by(&key) {
                Ok(i) => return self.entrys[i].value,
                Err(i) => self.entrys.insert(i, Entry { key, value }),
            },
            SetOption::UpdateOrInsert => match self.binary_search_by(&key) {
                Ok(i) => self.entrys[i].value = value,
                Err(_) => return self.insert(key, value, SetOption::FindOrInsert),
            },
        }
        value
    }

    pub fn is_full(&self) -> bool {
        self.entrys.len() >= Self::max_entrys_capacity()
    }

    pub fn split(&mut self) -> (Self, K) {
        let mut right = Self::new();

        let mid_point = Self::max_entrys_capacity() / 2;
        right.entrys = self.entrys.drain(mid_point..).collect();

        let mid = right.entrys[0].key;
        (right, mid)
    }

    pub fn find(&self, key: K) -> Option<&Entry<K, V, KS, VS>> {
        self.entrys.get(self.binary_search_by(&key).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::SetOption::*;
    type Leaf = super::Leaf<u64, u64, u16, 8, 8, 2, 4096>;

    #[test]
    fn to_from_bytes() {
        let mut leaf: Leaf = Leaf::new();
        for i in 0..255 {
            leaf.insert(i, i, UpdateOrInsert);
        }
        let buf = leaf.to_bytes();
        assert_eq!(buf.len(), 4086);
        assert_eq!(Leaf::from_bytes(&buf).to_bytes(), buf);
    }

    #[test]
    fn insert() {
        let mut leaf: Leaf = Leaf::new();

        for id in [1, 0, 5, 4, 2, 6, 3] {
            leaf.insert(id, 0, UpdateOrInsert);
        }
        let sorted_ids: Vec<_> = leaf.entrys.iter().map(|v| v.key).collect();
        assert!(sorted_ids.starts_with(&[0, 1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn split() {
        let mut left: Leaf = Leaf::new();

        assert_eq!(Leaf::max_entrys_capacity(), 255);
        
        for i in 1..=255 {
            left.insert(i, 0, UpdateOrInsert);
        }

        assert!(left.is_full());

        let (right, mid) = left.split();

        assert_eq!(mid, 128);

        let left_ids: Vec<_> = left.entrys.iter().map(|v| v.key).collect();
        let right_ids: Vec<_> = right.entrys.iter().map(|v| v.key).collect();

        assert_eq!(left_ids, (1..=127).collect::<Vec<u64>>());
        assert_eq!(right_ids, (128..=255).collect::<Vec<u64>>());
    }
}
