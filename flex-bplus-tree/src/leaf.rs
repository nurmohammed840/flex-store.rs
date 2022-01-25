use std::mem::replace;

use data_view::DataView;
use flex_page::PageNo;
use stack_array::Array;

use crate::entry::Key;
use SetOption::*;

pub enum SetOption {
    UpdateOrInsert,
    FindOrInsert,
}

pub struct Leaf<K, V, P, const PAGE_SIZE: usize>
where
    K: Key,
    V: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
{
    next:    P,
    prev:    P,
    entries: Array<(K, V), { (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE) }>,
}

impl<K, V, P, const PAGE_SIZE: usize> Leaf<K, V, P, PAGE_SIZE>
where
    K: Key + Ord,
    V: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
{
    pub fn new() -> Self { Self { next: P::new(0), prev: P::new(0), entries: Array::new() } }

    fn binary_search_by(&self, key: &K) -> Result<usize, usize> {
        self.entries.binary_search_by_key(&key, |entry| &entry.0)
    }

    pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> Option<V> {
        match self.binary_search_by(&key) {
            Ok(i) => Some(match opt {
                FindOrInsert => self.entries[i].1,
                UpdateOrInsert => replace(&mut self.entries[i].1, value),
            }),
            Err(i) => {
                self.entries.insert(i, (key, value));
                None
            }
        }
    }

    pub fn find(&self, key: K) -> Option<&(K, V)> {
        self.entries.get(self.binary_search_by(&key).ok()?)
    }

    pub fn split_at_mid(&mut self) -> (Self, K) {
        let mut right = Self::new();
        let mid_point = self.entries.len() / 2;
        right.entries.append(self.entries.drain(mid_point..).as_slice());
        let mid = right.entries[0].0;
        (right, mid)
    }

    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut buf = [0; PAGE_SIZE];
        let mut view = DataView::new(&mut buf[..]);

        view.write::<u8>(1); // Node Type
        view.write_slice(self.next.to_bytes());
        view.write_slice(self.prev.to_bytes());
        view.write(self.entries.len() as u16);

        for (key, value) in self.entries.iter() {
            view.write_slice(key.to_bytes());
            view.write_slice(value.to_bytes());
        }
        buf
    }

    pub fn from(mut view: DataView<&[u8]>) -> Self {
        let mut this = Self::new();
        
        this.next = P::from_bytes(view.read_buf());
        this.prev = P::from_bytes(view.read_buf());
        let len = view.read::<u16>();

        for _ in 0..len {
            let key = K::from_bytes(view.read_buf());
            let value = V::from_bytes(view.read_buf());
            this.entries.push((key, value));
        }
        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Leaf<const N: usize> = super::Leaf<u8, u16, u8, 4096>;

    #[test]
    fn check_capacity() {
        let leaf = super::Leaf::<u64, u16, u16, 4096>::new();
        assert_eq!(leaf.entries.capacity(), 408);
        
        let leaf = super::Leaf::<u32, u16, flex_page::U24, 4096>::new();
        assert_eq!(leaf.entries.capacity(), 681);
    }

    #[test]
    fn insert_and_find() {
        let mut leaf = Leaf::new();

        assert_eq!(leaf.insert(2, 2, UpdateOrInsert), None);
        assert_eq!(leaf.insert(1, 1, UpdateOrInsert), None);

        assert_eq!(leaf.insert(3, 33, FindOrInsert), None);
        assert_eq!(leaf.insert(4, 44, FindOrInsert), None);
        assert_eq!(leaf.insert(2, 22, UpdateOrInsert), Some(2));
        assert_eq!(leaf.insert(1, 11, UpdateOrInsert), Some(1));

        assert_eq!(leaf.insert(1, 111, FindOrInsert), Some(11));
        assert_eq!(leaf.insert(2, 222, FindOrInsert), Some(22));

        assert_eq!(leaf.find(3), Some(&(3, 33)));
        assert_eq!(leaf.find(4), Some(&(4, 44)));
        assert_eq!(leaf.find(99), None);

        let values = leaf.entries.iter().map(|(_, v)| *v).collect::<Vec<_>>();
        assert_eq!(values, [11, 22, 33, 44])
    }

    #[test]
    fn to_from_bytes() {
        let mut leaf = Leaf::new();
        leaf.next = 1;
        leaf.prev = 2;
        leaf.entries.push((3, 3));
        leaf.entries.push((4, 4));

        let bytes = leaf.to_bytes();
        let mut view = DataView::new(&bytes[..]);
        let _ = view.read::<u8>(); // Node Type

        let leaf2 = Leaf::from(view);
        assert_eq!(leaf2.next, 1);
        assert_eq!(leaf2.prev, 2);
        assert_eq!(leaf.entries[..], leaf2.entries[..]);
    }

    #[test]
    fn split_at_mid() {
        let mut left = Leaf::new();
        left.entries.append([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);

        let (right, mid) = left.split_at_mid();
        assert_eq!(left.entries.len(), 2);
        assert_eq!(right.entries.len(), 3);
        assert_eq!(mid, 3);
    }
}
