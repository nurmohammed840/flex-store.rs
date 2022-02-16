use bytes::{Buf, BufMut};

use crate::entry::Key;

pub struct Branch<K, const SIZE: usize> {
	pub keys: Vec<K>,
	pub childs: Vec<u16>,
}

impl<K: Key, const SIZE: usize> Branch<K, SIZE> {
	pub fn capacity() -> usize {
		// BlockSize - (Node type (1) + childs len (2))
		(SIZE - 3) / (K::SIZE + 2)
	}

	pub fn is_full(&self) -> bool {
		self.childs.len() >= Self::capacity()
	}

	pub fn new() -> Self {
		Self {
			keys: Vec::with_capacity(Self::capacity() - 1),
			childs: Vec::with_capacity(Self::capacity()),
		}
	}

	pub fn to_bytes(&self) -> [u8; SIZE] {
		let mut buf = [0; SIZE];
		let mut view = buf.as_mut();
		// Node type
		view.put_u8(1);
		// We don't need to write the `childs  length,
		// because it's always the same as the `keys` length + 1.
		view.put_u16_le(self.keys.len() as u16);
		self.keys.iter().for_each(|k| view.put(&k.to_bytes()[..]));
		self.childs.iter().for_each(|&c| view.put_u16_le(c));
		buf
	}

	pub fn from_bytes(mut bytes: &[u8]) -> Self {
		let keys_len = bytes.get_u16_le();
		let mut this = Self::new();

		for _ in 0..keys_len {
			this.keys.push(K::from_bytes(&bytes.copy_to_bytes(K::SIZE)));
		}
		for _ in 0..keys_len + 1 {
			this.childs.push(bytes.get_u16_le());
		}
		this
	}

	/// # Panic
	/// Panic if `childs` is empty,
	/// Make sure that `childs` has at least one element.
	pub fn insert(&mut self, index: usize, (k, n): (K, u16)) {
		self.keys.insert(index, k);
		self.childs.insert(index + 1, n);
	}

	pub fn lookup(&self, key: K) -> usize {
		let mut i = 0;
		while i < self.keys.len() && self.keys[i] <= key {
			i += 1;
		}
		i
	}

	pub fn create_root(key: K, left: u16, right: u16) -> Self {
		let mut branch = Self::new();
		branch.keys.push(key);
		branch.childs.push(left);
		branch.childs.push(right);
		branch
	}

	/// This function splits `Self` at the middle, and returns the other half. with reminder key.
	pub fn split_at_mid(&mut self) -> (Self, K) {
		let mid = self.childs.len() / 2;
		let keys = self.keys.drain(mid..).collect::<Vec<_>>();
		let childs = self.childs.drain(mid..).collect::<Vec<_>>();
		(Self { keys, childs }, self.keys.pop().unwrap())
	}

	/// Get a reference to the branch's childs.
	pub fn child_at(&self, i: usize) -> u16 {
		self.childs[i]
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	type Branch = super::Branch<u64, 4096>;

	#[test]
	fn check_capacity() {
		assert_eq!(Branch::capacity(), 409);
	}

	#[test]
	fn lookup() {
		let mut branch = Branch::new();
		branch.keys = [10, 20].to_vec();

		assert_eq!(branch.lookup(0), 0);
		assert_eq!(branch.lookup(9), 0);

		assert_eq!(branch.lookup(10), 1);
		assert_eq!(branch.lookup(19), 1);

		assert_eq!(branch.lookup(20), 2);
		assert_eq!(branch.lookup(100), 2);
	}

	fn test_byte_conversion(branch: &Branch) {
		let bytes = branch.to_bytes();
		let mut view = &bytes[..];

		assert_eq!(view.get_u8(), 1); // Node type

		let branch2 = Branch::from_bytes(view);

		assert_eq!(branch.keys, branch2.keys);
		assert_eq!(branch.childs, branch2.childs);
	}

	#[test]
	fn split_at_mid() {
		let mut branch = Branch::create_root(0, 0, 1);

		for i in 1..408 {
			branch.insert(i, (i as u64, i as u16 + 1));
		}

		assert!(branch.is_full());

		test_byte_conversion(&branch);

		let (other, remainder) = branch.split_at_mid();

		assert_eq!(branch.keys, (0..=202).collect::<Vec<_>>());
		assert_eq!(branch.childs, (0..=203).collect::<Vec<_>>());

		assert_eq!(remainder, 203);

		assert_eq!(other.keys, (204..=407).collect::<Vec<_>>());
		assert_eq!(other.childs, (204..=408).collect::<Vec<_>>());
	}
}
