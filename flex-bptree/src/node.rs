use bin_layout::{stack_array::ArrayBuf, Array, Cursor};

use super::*;
use crate::{branch::Branch, leaf::Leaf};

pub enum Node<K, V, const SIZE: usize> {
    Leaf(Leaf<K, V, SIZE>),
    Branch(Branch<K, SIZE>),
}

impl<K: Key, V: Key, const SIZE: usize> Node<K, V, SIZE> {
    pub fn encode(self) -> ArrayBuf<u8, SIZE> {
        let mut arr = ArrayBuf::new();
        match self {
            Node::Leaf(leaf) => {
                arr.push(0);
                leaf.encoder(&mut arr);
            }
            Node::Branch(branch) => {
                arr.push(1);
                branch.encoder(&mut arr)
            }
        }
        arr
    }

    pub fn decoder(buf: &[u8]) -> Self {
        let mut c = Cursor::new(buf);
        match (u8::decoder(&mut c) as Result<u8, ()>).unwrap() {
            0 => Node::Leaf(Leaf::decoder(&mut c).unwrap()),
            1 => Node::Branch(Branch::decoder(&mut c).unwrap()),
            _ => panic!("invalid node type"),
        }
    }
}