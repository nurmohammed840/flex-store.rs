use crate::entry::*;
use byte_seeker::ByteSeeker;
use flex_page::PageNo;

pub enum SetOption {
    UpdateOrInsert,
    FindOrInsert,
}

pub struct Leaf<K, V, P, const X: usize, const Y: usize, const Z: usize, const PAGE_SIZE: usize> {
    pub left: P,
    pub right: P,
    pub entrys: Vec<Entry<K, V, X, Y>>,
}

impl<
        K,
        V,
        P,
        const KEY_SIZE: usize,
        const VALUE_SIZE: usize,
        const PAGE_NO_SIZE: usize,
        const PAGE_SIZE: usize,
    > Leaf<K, V, P, KEY_SIZE, VALUE_SIZE, PAGE_NO_SIZE, PAGE_SIZE>
where
    K: Ord + Key<KEY_SIZE>,
    V: Key<VALUE_SIZE>,
    P: PageNo<PAGE_NO_SIZE>,
{
    pub fn max_entrys_capacity() -> usize {
        // 7 = 1 node type + 2 node len + 2 `left` + 2 `right` + 2 entry len
        (PAGE_SIZE - 9) / (KEY_SIZE + VALUE_SIZE)
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
            let bytes = byte_seeker.octets(KEY_SIZE + VALUE_SIZE);
            leaf.entrys
                .push(Entry::<K, V, KEY_SIZE, VALUE_SIZE>::from_bytes(&bytes));
        }
        leaf
    }

    fn new() -> Self {
        Self {
            entrys: Vec::with_capacity(Self::max_entrys_capacity()),
            left: P::default(),
            right: P::default(),
        }
    }

    fn binary_search(&self, id: &K) -> Result<usize, usize> {
        self.entrys.binary_search_by_key(&id, |e| &e.key)
    }

    /// Insert and sort `entrys`
    pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> V {
        match opt {
            SetOption::FindOrInsert => match self.binary_search(&key) {
                Ok(i) => return self.entrys[i].value,
                Err(i) => self.entrys.insert(i, Entry { key, value }),
            },
            SetOption::UpdateOrInsert => match self.binary_search(&key) {
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

    pub fn find(&self, key: K) -> Option<&Entry<K, V, KEY_SIZE, VALUE_SIZE>> {
        self.entrys.get(self.binary_search(&key).ok()?)
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
