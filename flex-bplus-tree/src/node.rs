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
                buf[0] = 1;
                buf[1..=bytes.len()].copy_from_slice(&bytes);
            }
            Node::Leaf(leaf) => {
                let bytes = leaf.to_bytes();
                buf[1..=bytes.len()].copy_from_slice(&bytes);
            }
        }
        buf
    }
    fn from_bytes(bytes: [u8; PAGE_SIZE]) -> Self {
        match bytes[0] {
            0 => Node::Leaf(Leaf::from_bytes(&bytes[1..])),
            _ => Node::Branch(Branch::from_bytes(&bytes[1..])),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::leaf::SetOption::*;

    type Node = super::Node<u64, u64, u16, 8, 8, 2, 4096>;
    type Leaf = crate::leaf::Leaf<u64, u64, u16, 8, 8, 2, 4096>;

    const BYTES: [u8; 4096] = [0; 4096];

    #[test]
    fn default_node() {
        let is_leaf_node = match Node::from_bytes(BYTES) {
            Node::Branch(_) => false,
            Node::Leaf(_) => true,
        };
        assert!(is_leaf_node);
    }

    #[test]
    fn to_from_bytes() {
        let mut left: Leaf = Leaf::new();
        for i in 1..=255 {
            left.insert(i, 0, UpdateOrInsert);
        }
        let bytes = Node::Leaf(left).to_bytes();
        assert_eq!(bytes, Node::from_bytes(bytes).to_bytes());
    }
}
