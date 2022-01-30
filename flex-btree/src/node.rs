use bytes::Buf;
use flex_page::PageNo;

use crate::branch::Branch;
use crate::entry::Key;
use crate::leaf::Leaf;

pub enum Node<K, V, N, const SIZE: usize> {
	// This is default node type
	Leaf(Leaf<K, V, N, SIZE>),
	Branch(Branch<K, N, SIZE>),
}

impl<K: Key, V: Key, N: PageNo, const SIZE: usize> Node<K, V, N, SIZE> {
	pub fn to_bytes(&self) -> [u8; SIZE] {
		match self {
			Node::Leaf(leaf) => leaf.to_bytes(),
			Node::Branch(branch) => branch.to_bytes(),
		}
	}

	pub fn from_bytes(bytes: [u8; SIZE]) -> Self {
		let mut view = &bytes[..];
		match view.get_u8() {
			0 => Node::Leaf(Leaf::from_bytes(view)),
			1 => Node::Branch(Branch::from_bytes(view)),
            _ => panic!("Invalid Node Type")
		}
	}
}