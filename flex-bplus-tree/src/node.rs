use crate::{branch::Branch, entry::Key, leaf::Leaf};

pub enum Node<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> {
    Leaf(Leaf<K, V, X, Y, PAGE_SIZE>),
    Branch(Branch<K, X, PAGE_SIZE>),
}

impl<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> Node<K, V, X, Y, PAGE_SIZE>
where
    K: Key<X> + Ord,
    V: Key<Y>,
{
    fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut buf = [0; PAGE_SIZE];
        match self {
            Node::Leaf(leaf) => {
                buf[0] = 1;
                buf[1..].copy_from_slice(&leaf.to_bytes());
            }
            Node::Branch(branch) => {
                buf[0] = 0;
                buf[1..].copy_from_slice(&branch.to_bytes());
            }
        }
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        todo!()
    }
}
