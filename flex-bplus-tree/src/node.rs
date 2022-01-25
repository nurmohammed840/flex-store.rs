use data_view::DataView;
use flex_page::PageNo;
use stack_array::Array;

use crate::branch::Branch;
use crate::entry::Key;
use crate::leaf::Leaf;

pub enum Node<K, V, P, const PAGE_SIZE: usize>
where
    K: Key,
    V: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
{
    // Branch(Branch<K, PAGE_SIZE>),
    Leaf(Leaf<K, V, P, PAGE_SIZE>),
    Branch(Branch<K, P, PAGE_SIZE>),
}

impl<K, V, P, const PAGE_SIZE: usize> Node<K, V, P, PAGE_SIZE>
where
    K: Key + Ord,
    V: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
{
    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        match self {
            Node::Branch(branch) => branch.to_bytes(),
            Node::Leaf(leaf) => leaf.to_bytes(),
        }
    }

    pub fn from_bytes(bytes: [u8; PAGE_SIZE]) -> Self {
        let mut view = DataView::new(&bytes[..]);
        match view.read::<u8>() {
            0 => Node::Branch(Branch::from(view)),
            _ => Node::Leaf(Leaf::from(view)),
        }
    }
}

// ====================================================================================================

// #[cfg(test)]
// mod tests {
//     use crate::leaf::SetOption::*;

//     type Node = super::Node<u64, u64, u16, 8, 8, 2, 4096>;
//     type Leaf = crate::leaf::Leaf<u64, u64, u16, 8, 8, 2, 4096>;

//     const BYTES: [u8; 4096] = [0; 4096];

//     #[test]
//     fn default_node() {
//         let is_leaf_node = match Node::from_bytes(BYTES) {
//             Node::Branch(_) => false,
//             Node::Leaf(_) => true,
//         };
//         assert!(is_leaf_node);
//     }

//     #[test]
//     fn to_from_bytes() {
//         let mut left: Leaf = Leaf::new();
//         for i in 1..=255 {
//             left.insert(i, 0, UpdateOrInsert);
//         }
//         let bytes = Node::Leaf(left).to_bytes();
//         assert_eq!(bytes, Node::from_bytes(bytes).to_bytes());
//     }
// }
