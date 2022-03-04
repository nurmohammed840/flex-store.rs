use super::*;

pub enum Node<K, V, const SIZE: usize> {
	// This is default node type
	Leaf(Leaf<K, V, SIZE>),
	Branch(Branch<K, SIZE>),
}

impl<K: Key, V: Key, const SIZE: usize> Node<K, V, SIZE> {
	pub fn from_bytes(bytes: [u8; SIZE]) -> Self {
		match bytes[0] {
			0 => Node::Leaf(Leaf::from_bytes(bytes)),
			1 => Node::Branch(Branch::from_bytes(bytes)),
			_ => panic!("Invalid Node Type"),
		}
	}
}
