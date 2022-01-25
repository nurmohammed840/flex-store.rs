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
    childs: Array<P, { (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) }>,
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
        view.write::<u8>(0);
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

    // pub fn insert(&mut self, index: usize, key: K, child: P) {
    //     self.keys.insert(index, key);
    //     self.childs.insert(index + 1, child);
    // }
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

    // #[test]
    // fn check_to_bytes() {
    //     let mut branch = Branch::<u64, u16, 4096>::new();
    //     branch.insert(0, 1, 2);
    //     branch.insert(1, 3, 4);
    //     branch.insert(2, 5, 6);
    //     println!("{:#?}", branch.keys);
    //     println!("{:#?}", branch.childs);
    // }
}

// ===========================================================================================

// impl<K, P, const KS: usize, const PS: usize, const PAGE_SIZE: usize> Branch<K, P, KS, PS, PAGE_SIZE>
// where
//     K: PartialOrd + Key<KS>,
//     P: PageNo<PS>,
// {
//     pub fn create_root(key: K, left: P, right: P) -> Self {
//         let mut branch = Branch::new();
//         branch.keys.push(key);
//         branch.childs.push(left);
//         branch.childs.push(right);
//         branch
//     }

//     /// -> index
//     pub fn lookup(&self, key: K) -> usize {
//         let mut i = 0;
//         for &_key in &self.keys {
//             if key < _key {
//                 return i;
//             }
//             i += 1;
//         }
//         i
//     }

//     /// ### Panic: যদি `branch.childs`-এ কোনো উপাদান না থাকে, নিশ্চিত করুন যে `branch.childs`-এ অন্তত একটি উপাদান রয়েছে.
//     pub fn update(&mut self, index: usize, (mid, page_no): (K, P)) {
//         self.keys.insert(index, mid);
//         self.childs.insert(index + 1, page_no);
//     }

//     pub fn split(&mut self) -> (Self, K) {
//         let mid_point = Self::max_keys_capacity() / 2;
//         let right = Self {
//             keys: self.keys.drain(mid_point..).collect(),
//             childs: self.childs.drain(mid_point..).collect(),
//         };
//         (right, self.keys.pop().unwrap())
//     }
// }

// #[cfg(test)]
// mod tests {
//     type Branch = super::Branch<u64, u16, 8, 2, 4096>;

//     #[test]
//     fn lookup() {
//         let mut branch: Branch = Branch::new();
//         branch.keys = vec![10u64, 15, 20];

//         assert_eq!(branch.lookup(10), 1);
//         assert_eq!(branch.lookup(15), 2);
//         assert_eq!(branch.lookup(20), 3);

//         assert_eq!(branch.lookup(0), 0);
//         assert_eq!(branch.lookup(9), 0);
//         assert_eq!(branch.lookup(11), 1);
//         assert_eq!(branch.lookup(14), 1);
//         assert_eq!(branch.lookup(16), 2);
//         assert_eq!(branch.lookup(19), 2);
//         assert_eq!(branch.lookup(21), 3);
//         assert_eq!(branch.lookup(100), 3);
//     }

//     #[test]
//     fn split() {
//         let mut left: Branch = Branch::create_root(0, 0, 1);

//         for i in 1..Branch::max_keys_capacity() {
//             left.update(i, (i as u64, i as u16 + 1));
//         }

//         assert!(left.is_full());

//         // ------------------  to/from byte test ------------------
//         let bytes = left.to_bytes();
//         assert_eq!(bytes.len(), 4084);
//         assert_eq!(Branch::from_bytes(&bytes).to_bytes(), bytes);
//         // --------------------------------------------------------

//         let (right, mid) = left.split();

//         assert_eq!(mid, 203);

//         assert_eq!(left.keys, (0..=202).collect::<Vec<u64>>());
//         assert_eq!(right.keys, (204..=407).collect::<Vec<u64>>());

//         assert_eq!(left.childs, (0..=203).collect::<Vec<u16>>());
//         assert_eq!(right.childs, (204..=408).collect::<Vec<u16>>());
//     }
// }
