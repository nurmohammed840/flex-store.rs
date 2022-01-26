use data_view::DataView;
use flex_page::PageNo;
use stack_array::Array;

use crate::entry::Key;

pub struct Branch<K, P, const PAGE_SIZE: usize>
where
    K: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
{
    keys:   Array<K, { (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1 }>,
    pub childs: Array<P, { (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) }>,
}

impl<K, P, const PAGE_SIZE: usize> Branch<K, P, PAGE_SIZE>
where
    K: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
{
    pub fn new() -> Self { Self { keys: Array::new(), childs: Array::new() } }

    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut buf = [0; PAGE_SIZE];
        let mut view = DataView::new(&mut buf[..]);
        // Node type
        view.write::<u8>(1);
        // We don't need to write the `childs  length,
        // because it's always the same as the `keys` length + 1.
        view.write(self.keys.len() as u16);
        self.keys.iter().for_each(|k| view.write_slice(k.to_bytes()));
        self.childs.iter().for_each(|c| view.write_slice(c.to_bytes()));
        buf
    }

    pub fn from(mut view: DataView<&[u8]>) -> Self {
        let keys_len = view.read::<u16>();
        let mut this = Self::new();
        for _ in 0..keys_len {
            this.keys.push(K::from_bytes(view.read_buf()));
        }
        for _ in 0..keys_len + 1 {
            this.childs.push(P::from_bytes(view.read_buf()));
        }
        this
    }

    /// # Panic
    /// Panic if `childs` is empty,
    /// Make sure that `childs` has at least one element.
    pub fn insert(&mut self, index: usize, key: K, child: P) {
        self.keys.insert(index, key);
        self.childs.insert(index + 1, child);
    }

    pub fn lookup(&self, key: K) -> usize {
        let mut i = 0;
        let len = self.keys.len();
        while i < len && self.keys[i] <= key {
            i += 1;
        }
        i
    }

    pub fn create_root(key: K, left: P, right: P) -> Self {
        let mut branch = Self::new();
        branch.keys.push(key);
        branch.childs.push(left);
        branch.childs.push(right);
        branch
    }

    /// This function splits `Self` at the middle, and returns the other half. with reminder key.
    pub fn split_at_mid(&mut self) -> (Self, K) {
        let mid = self.keys.len() / 2;
        let mut other = Self::new();
        other.keys.append(self.keys.drain(mid..).as_slice());
        other.childs.append(self.childs.drain(mid..).as_slice());
        (other, self.keys.pop())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_capacity() {
        let branch = Branch::<u64, u16, 4096>::new();
        assert_eq!(branch.keys.capacity(), 408);
        assert_eq!(branch.childs.capacity(), 409);

        let branch = Branch::<[u8; 16], u32, 4096>::new();
        assert_eq!(branch.keys.capacity(), 203);
        assert_eq!(branch.childs.capacity(), 204);

        let branch = Branch::<u32, flex_page::U24, 4096>::new();
        assert_eq!(branch.keys.capacity(), 583);
        assert_eq!(branch.childs.capacity(), 584);
    }

    #[test]
    fn lookup() {
        let mut branch = Branch::<u64, u16, 4096>::new();
        branch.keys.append([10, 20]);

        assert_eq!(branch.lookup(0), 0);
        assert_eq!(branch.lookup(9), 0);

        assert_eq!(branch.lookup(10), 1);
        assert_eq!(branch.lookup(19), 1);

        assert_eq!(branch.lookup(20), 2);
        assert_eq!(branch.lookup(100), 2);
    }

    fn test_byte_conversion(branch: &Branch<u64, u16, 4096>) {
        let bytes = branch.to_bytes();
        let mut view = DataView::new(&bytes[..]);

        assert_eq!(view.read::<u8>(), 1); // Node type

        let branch2 = Branch::<u64, u16, 4096>::from(view);

        assert_eq!(branch2.keys[..], branch.keys[..]);
        assert_eq!(branch2.childs[..], branch.childs[..]);
    }

    #[test]
    fn split_at_mid() {
        let mut branch = Branch::<u64, u16, 4096>::create_root(0, 0, 1);

        for i in 1..branch.keys.capacity() {
            branch.insert(i, i as u64, i as u16 + 1);
        }

        test_byte_conversion(&branch);

        assert!(branch.keys.is_full());
        assert!(branch.childs.is_full());

        let (other, remainder) = branch.split_at_mid();

        assert_eq!(branch.keys[..], (0..=202).collect::<Vec<u64>>());
        assert_eq!(branch.childs[..], (0..=203).collect::<Vec<u16>>());
        
        assert_eq!(remainder, 203);

        assert_eq!(other.keys[..], (204..=407).collect::<Vec<u64>>());
        assert_eq!(other.childs[..], (204..=408).collect::<Vec<u16>>());
    }
}
