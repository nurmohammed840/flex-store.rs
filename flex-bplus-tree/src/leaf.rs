use crate::{entry, entry::Entry};
use byte_seeker::ByteSeeker;

pub enum SetOption {
    UpdateOrInsert,
    FindOrInsert,
}

pub struct Leaf<
    Key,
    Value,
    PageNo,
    const KEY: usize,
    const VALUE: usize,
    const PAGE_NO: usize,
    const PAGE_SIZE: usize,
> {
    pub left: PageNo,
    pub right: PageNo,
    pub entrys: Vec<Entry<Key, Value, KEY, VALUE>>,
}

impl<
        Key,
        Value,
        PageNo,
        const KEY: usize,
        const VALUE: usize,
        const PAGE_NO: usize,
        const PAGE_SIZE: usize,
    > Leaf<Key, Value, PageNo, KEY, VALUE, PAGE_NO, PAGE_SIZE>
where
    Key: Ord + entry::Key<KEY>,
    Value: entry::Key<VALUE>,
    PageNo: flex_page::PageNo<PAGE_NO>,
{
    pub fn max_entrys_capacity() -> usize {
        //  node type + node len + ('left' + 'right') + entry len
        let margin = 1 + 2 + (PAGE_NO * 2) + 2;
        (PAGE_SIZE - margin) / (KEY + VALUE)
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

        leaf.left = PageNo::from_bytes(byte_seeker.buf());
        leaf.right = PageNo::from_bytes(byte_seeker.buf());

        let len = u16::from_le_bytes(byte_seeker.buf());

        for _ in 0..len {
            let bytes = byte_seeker.octets(KEY + VALUE);
            leaf.entrys
                .push(Entry::<Key, Value, KEY, VALUE>::from_bytes(&bytes));
        }
        leaf
    }

    fn new() -> Self {
        Self {
            entrys: Vec::with_capacity(Self::max_entrys_capacity()),
            left: PageNo::default(),
            right: PageNo::default(),
        }
    }

    fn binary_search_by(&self, key: &Key) -> Result<usize, usize> {
        self.entrys.binary_search_by_key(&key, |entry| &entry.key)
    }

    /// Insert and sort `entrys`
    pub fn insert(&mut self, key: Key, value: Value, opt: SetOption) -> Value {
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

    pub fn split(&mut self) -> (Self, Key) {
        let mut right = Self::new();

        let mid_point = Self::max_entrys_capacity() / 2;
        right.entrys = self.entrys.drain(mid_point..).collect();

        let mid = right.entrys[0].key;
        (right, mid)
    }

    pub fn find(&self, key: Key) -> Option<&Entry<Key, Value, KEY, VALUE>> {
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
