use bytes::{Buf, BufMut};
use flex_page::PageNo;

use crate::entry::Key;

#[derive(Debug)]
pub struct Branch<K, N, const SIZE: usize> {
	pub keys: Vec<K>,
	pub childs: Vec<N>,
}

impl<K: Key, N: PageNo, const SIZE: usize> Branch<K, N, SIZE> {
	pub fn capacity() -> usize {
		(SIZE - (1 + 2)) / (K::SIZE + N::SIZE)
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
		self.childs.iter().for_each(|c| view.put(&c.to_bytes()[..]));
		buf
	}

	pub fn from_bytes(mut bytes: &[u8]) -> Self {
		let keys_len = bytes.get_u16_le();
		let mut this = Self::new();

		this.keys.reserve(keys_len as usize);
		this.childs.reserve((keys_len + 1) as usize);

		for _ in 0..keys_len {
			this.keys.push(K::from_bytes(&bytes.copy_to_bytes(K::SIZE)));
		}
		for _ in 0..keys_len + 1 {
			this.childs
				.push(N::from_bytes(&bytes.copy_to_bytes(N::SIZE)));
		}
		this
	}

	/// # Panic
	/// Panic if `childs` is empty,
	/// Make sure that `childs` has at least one element.
	pub fn insert(&mut self, index: usize, (k, n): (K, N)) {
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

	pub fn create_root(key: K, left: N, right: N) -> Self {
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
	pub fn child_at(&self, i: usize) -> N {
		self.childs[i]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_capacity() {
		assert_eq!(Branch::<u64, u16, 4096>::capacity(), 409);
		assert_eq!(Branch::<[u8; 16], u32, 4096>::capacity(), 204);
		assert_eq!(Branch::<u32, flex_page::U24, 4096>::capacity(), 584);
	}

	#[test]
	fn lookup() {
		let mut branch = Branch::<u64, u16, 4096>::new();
		branch.keys = [10, 20].to_vec();

		assert_eq!(branch.lookup(0), 0);
		assert_eq!(branch.lookup(9), 0);

		assert_eq!(branch.lookup(10), 1);
		assert_eq!(branch.lookup(19), 1);

		assert_eq!(branch.lookup(20), 2);
		assert_eq!(branch.lookup(100), 2);
	}

	fn test_byte_conversion(branch: &Branch<u64, u16, 4096>) {
		let bytes = branch.to_bytes();
		let mut view = &bytes[..];

		assert_eq!(view.get_u8(), 1); // Node type

		let branch2 = Branch::<u64, u16, 4096>::from_bytes(view);

		assert_eq!(branch.keys, branch2.keys);
		assert_eq!(branch.childs, branch2.childs);
	}

	#[test]
	fn split_at_mid() {
		let mut branch = Branch::<u64, u16, 4096>::create_root(0, 0, 1);

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

#[test]
fn test_name() {
	let mut branch = Branch::<u64, u16, 4096>::create_root(2, 0, 1);
	branch.insert(1, (3, 2));
	branch.insert(2, (4, 3));
	// branch.insert(2, (3, 4));
	// branch.insert(3, (5, 6));
	println!("Original : {:?}", branch);

	let a = branch.split_at_mid();
	println!("{:?}", a.1);
	println!("{:?}", branch);
	println!("{:?}", a.0);
}
// Original : Branch { keys: [0, 1], childs: [0, 1, 2] }
