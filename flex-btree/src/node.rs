use bytes::Buf;

use crate::branch::Branch;
use crate::entry::Key;
use crate::leaf::Leaf;

pub enum Node<K, V, const SIZE: usize> {
	// This is default node type
	Leaf(Leaf<K, V, SIZE>),
	Branch(Branch<K, SIZE>),
}

impl<K: Key, V: Key, const SIZE: usize> Node<K, V, SIZE> {
	pub fn from_bytes(bytes: [u8; SIZE]) -> Self {
		let mut view = &bytes[..];
		match view.get_u8() {
			0 => Node::Leaf(Leaf::from_bytes(view)),
			1 => Node::Branch(Branch::from_bytes(view)),
			_ => panic!("Invalid Node Type"),
		}
	}
}
