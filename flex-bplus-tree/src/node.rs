use bytes::Buf;
use page::PageNo;

use crate::branch::Branch;
use crate::entry::Key;
use crate::leaf::Leaf;

pub enum Node<K, V, P, const PAGE_SIZE: usize> {
    // This is default node type
    Leaf(Leaf<K, V, P, PAGE_SIZE>),
    Branch(Branch<K, P, PAGE_SIZE>),
}

impl<K: Key, V: Key, P: PageNo, const PAGE_SIZE: usize> Node<K, V, P, PAGE_SIZE> {
    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        match self {
            Node::Branch(branch) => branch.to_bytes(),
            Node::Leaf(leaf) => leaf.to_bytes(),
        }
    }

    pub fn from_bytes(bytes: [u8; PAGE_SIZE]) -> Self {
        let mut view = &bytes[..];
        match view.get_u8() {
            1 => Node::Branch(Branch::from_bytes(view)),
            _ => Node::Leaf(Leaf::from_bytes(view)),
        }
    }
}
