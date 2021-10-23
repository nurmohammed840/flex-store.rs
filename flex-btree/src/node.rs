use crate::{branch::Branch, leaf::Leaf};
use std::mem::transmute;

#[repr(C, u64)]
pub enum Node {
    #[allow(dead_code)]
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn to_bytes(self) -> [u8; 4096] {
        unsafe { transmute::<Self, [u8; 4096]>(self) }
    }
    pub fn from_bytes(bytes: [u8; 4096]) -> Self {
        unsafe { transmute::<[u8; 4096], Self>(bytes) }
    }

    pub fn get_leaf(self) -> Option<Leaf> {
        if let Node::Leaf(leaf) = self {
            Some(leaf)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::leaf::SetOption;

    use super::*;
    const BYTES: [u8; 4096] = [0; 4096];

    #[test]
    fn default_node() {
        let is_leaf_node = match Node::from_bytes(BYTES) {
            Node::Leaf(_) => true,
            _ => false,
        };
        assert!(is_leaf_node);
    }

    #[test]
    fn perform_transmute() {
        let dummy_leaf_node = || {
            let mut leaf = Leaf::new();
            for i in 1u8..=255 {
                leaf.set_and_sort_entry(i as u64, [i; 8], SetOption::UpdateOrInsert);
            }
            Node::Leaf(leaf).to_bytes()
        };
        assert_eq!(dummy_leaf_node(), dummy_leaf_node());
    }
}
