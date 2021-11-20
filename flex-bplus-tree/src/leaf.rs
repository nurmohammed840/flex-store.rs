use std::mem;

use crate::entry::*;
use byte_seeker::ByteSeeker;

pub enum SetOption {
    UpdateOrInsert,
    FindOrInsert,
}

pub struct Leaf<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> {
    pub left: u16,
    pub right: u16,
    pub entrys: Vec<Entry<K, V, X, Y>>,
}

impl<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> Leaf<K, V, X, Y, PAGE_SIZE>
where
    K: Ord + Key<X>,
    V: Key<Y>,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PAGE_SIZE);

        bytes.extend_from_slice(&self.left.to_le_bytes());
        bytes.extend_from_slice(&self.right.to_le_bytes());

        let len: u16 = self.entrys.len().try_into().unwrap();
        bytes.extend_from_slice(&len.to_le_bytes());

        for entry in self.entrys.iter() {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut byte_seeker = ByteSeeker::new(bytes);
        let mut leaf = Self::new();

        leaf.left = u16::from_le_bytes(byte_seeker.buf_unwrap());
        leaf.right = u16::from_le_bytes(byte_seeker.buf_unwrap());

        let len = u16::from_le_bytes(byte_seeker.buf_unwrap());

        for _ in 0..len {
            let bytes = byte_seeker.octets_unwrap(X + Y);
            leaf.entrys.push(Entry::<K, V, X, Y>::from_bytes(&bytes));
        }
        leaf
    }

    fn new() -> Self {
        Self {
            entrys: Vec::with_capacity(Self::max_entrys_capacity()),
            left: 0,
            right: 0,
        }
    }

    pub fn max_entrys_capacity() -> usize {
        // 7 = 1 byte node type + 2 bytes entry len + 2 bytes `left_child` size +  2 bytes `right_child` size
        (PAGE_SIZE - 7) / mem::size_of::<Entry<K, V, X, Y>>()
    }

    fn binary_search(&self, id: K) -> Result<usize, usize> {
        self.entrys.binary_search_by_key(&id, |e| e.key)
    }

    /// Insert and sort `entrys`
    pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> V {
        match opt {
            SetOption::FindOrInsert => match self.binary_search(key) {
                Ok(i) => return self.entrys[i].value,
                Err(i) => self.entrys.insert(i, Entry { key, value }),
            },
            SetOption::UpdateOrInsert => match self.binary_search(key) {
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

    pub fn find(&self, id: K) -> Option<&Entry<K, V, X, Y>> {
        self.entrys.get(self.binary_search(id).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut leaf: Leaf<u64, u64, 8, 8, 4096> = Leaf::new();

        for id in [1, 0, 5, 4, 2, 6, 3] {
            leaf.insert(id, 0, SetOption::UpdateOrInsert);
        }
        let sorted_ids: Vec<_> = leaf.entrys.iter().map(|v| v.key).collect();
        assert!(sorted_ids.starts_with(&[0, 1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn split() {
        let mut left: Leaf<u64, u64, 8, 8, 4096> = Leaf::new();

        for i in 1..=255 {
            left.insert(i, 0, SetOption::UpdateOrInsert);
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
