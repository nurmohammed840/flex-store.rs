use flex_page::PageNo;

use crate::{branch::Branch, entry::Key, leaf::Leaf};

pub enum Node<
    K,
    V,
    P,
    const KEY_SIZE: usize,
    const VALUE_SIZE: usize,
    const PAGE_NO_SIZE: usize,
    const PAGE_SIZE: usize,
> {
    Leaf(Leaf<K, V, P, KEY_SIZE, VALUE_SIZE, PAGE_NO_SIZE, PAGE_SIZE>),
    Branch(Branch<K, P, KEY_SIZE, PAGE_NO_SIZE, PAGE_SIZE>),
}

impl<
        K,
        V,
        P,
        const KEY_SIZE: usize,
        const VALUE_SIZE: usize,
        const PAGE_NO_SIZE: usize,
        const PAGE_SIZE: usize,
    > Node<K, V, P, KEY_SIZE, VALUE_SIZE, PAGE_NO_SIZE, PAGE_SIZE>
where
    K: Key<KEY_SIZE> + Ord,
    V: Key<VALUE_SIZE>,
    P: PageNo<PAGE_NO_SIZE>,
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
