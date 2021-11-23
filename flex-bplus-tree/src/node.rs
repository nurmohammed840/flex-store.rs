use crate::{branch::Branch, entry, leaf::Leaf};

pub enum Node<
    Key,
    Value,
    PageNo,
    const KEY: usize,
    const VALUE: usize,
    const PAGE_NO: usize,
    const PAGE_SIZE: usize,
> {
    Leaf(Leaf<Key, Value, PageNo, KEY, VALUE, PAGE_NO, PAGE_SIZE>),
    Branch(Branch<Key, PageNo, KEY, PAGE_NO, PAGE_SIZE>),
}

impl<
        Key,
        Value,
        PageNo,
        const KEY: usize,
        const VALUE: usize,
        const PAGE_NO: usize,
        const PAGE_SIZE: usize,
    > Node<Key, Value, PageNo, KEY, VALUE, PAGE_NO, PAGE_SIZE>
where
    Key: entry::Key<KEY> + Ord,
    Value: entry::Key<VALUE>,
    PageNo: flex_page::PageNo<PAGE_NO>,
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
