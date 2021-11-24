use flex_page::PageNo;

use crate::{branch::Branch, entry::Key, leaf::Leaf};

pub enum Node<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize> {
    Leaf(Leaf<K, V, P, KS, VS, PS, PAGE_SIZE>),
    Branch(Branch<K, P, KS, PS, PAGE_SIZE>),
}

impl<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
    Node<K, V, P, KS, VS, PS, PAGE_SIZE>
where
    K: Key<KS> + Ord,
    V: Key<VS>,
    P: PageNo<PS>,
{
    fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut buf = [0; PAGE_SIZE];
        match self {
            Node::Branch(branch) => {
                let bytes = branch.to_bytes();
                buf[1..bytes.len()].copy_from_slice(&bytes);
            }
            Node::Leaf(leaf) => {
                buf[0] = 1;
                let bytes = leaf.to_bytes();
                buf[1..bytes.len()].copy_from_slice(&bytes);
            }
        }
        buf
    }
    fn from_bytes(bytes: [u8; PAGE_SIZE]) -> Self {
        match bytes[0] {
            0 => Node::Branch(Branch::from_bytes(&bytes[1..])),
            _ => Node::Leaf(Leaf::from_bytes(&bytes[1..])),
        }
    }
}
