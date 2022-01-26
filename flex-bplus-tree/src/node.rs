use data_view::DataView;
use flex_page::PageNo;

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
    // This is default node type
    Leaf(Leaf<K, V, P, PAGE_SIZE>), 
    Branch(Branch<K, P, PAGE_SIZE>),
}

impl<K, V, P, const PAGE_SIZE: usize> Node<K, V, P, PAGE_SIZE>
where
    K: Key,
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
            1 => Node::Branch(Branch::from(view)),
            _ => Node::Leaf(Leaf::from(view)),
        }
    }
}