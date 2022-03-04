use std::mem::replace;

use bytes::{Buf, BufMut};

use crate::entry::Key;
use SetOption::*;

#[derive(Debug, Clone)]
pub enum SetOption {
	UpdateOrInsert,
	FindOrInsert,
}

pub struct Leaf<K, V, const SIZE: usize> {
	pub next: u16,
	pub prev: u16,
	pub entries: Vec<(K, V)>,
}

impl<K: Key, V: Key, const SIZE: usize> Leaf<K, V, SIZE> {
	pub fn capacity() -> usize {
		// BlockSize - (Node type (1) + next (2) + prev (2) + entries len (2))
		(SIZE - 7) / (K::SIZE + V::SIZE)
	}

	pub fn is_half_full(&self) -> bool {
		self.entries.len() > (Self::capacity() / 2)
	}

	pub fn new() -> Self {
		Self {
			next: 0,
			prev: 0,
			entries: Vec::with_capacity(Self::capacity()),
		}
	}

	pub fn is_full(&self) -> bool {
		self.entries.len() >= Self::capacity()
	}

	pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> Option<V> {
		match self.binary_search(&key) {
			Ok(i) => Some(match opt {
				FindOrInsert => self.entries[i].1,
				UpdateOrInsert => replace(&mut self.entries[i].1, value),
			}),
			Err(i) => {
				self.entries.insert(i, (key, value));
				None
			}
		}
	}

	/// This function splits `Self` at the middle and returns the right half.
	pub fn split_at_mid(&mut self) -> (Self, K) {
		let mut other = Self::new();
		let mid_point = self.entries.len() / 2;
		other.entries = self.entries.drain(mid_point..).collect();
		let mid = other.entries[0].0;
		(other, mid)
	}

	pub fn to_bytes(&self) -> [u8; SIZE] {
		let mut buf = [0; SIZE];
		let mut view = buf.as_mut();

		view.put_u8(0); // Node Type
		view.put_u16_le(self.next);
		view.put_u16_le(self.prev);
		view.put_u16_le(self.entries.len() as u16);

		for (key, value) in self.entries.iter() {
			view.put(&key.to_bytes()[..]);
			view.put(&value.to_bytes()[..]);
		}
		buf
	}

	pub fn from_bytes(bytes: [u8; SIZE]) -> Self {
		let mut this = Self::new();
		let mut view = bytes.as_ref();

		let _ = view.get_u8(); // Node Type
		this.next = view.get_u16_le();
		this.prev = view.get_u16_le();
		let len = view.get_u16_le();

		for _ in 0..len {
			let key = K::from_bytes(&view.copy_to_bytes(K::SIZE));
			let value = V::from_bytes(&view.copy_to_bytes(V::SIZE));
			this.entries.push((key, value));
		}
		this
	}

	pub fn binary_search(&self, key: &K) -> Result<usize, usize> {
		self.entries
			.binary_search_by(|(k, _)| k.partial_cmp(key).expect("Key can't be `NaN`"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	type Leaf<const N: usize> = super::Leaf<u64, u16, 4096>;

	#[test]
	fn check_capacity() {
		assert_eq!(Leaf::capacity(), 408);
	}

	#[test]
	fn insert_and_find() {
		let mut leaf = Leaf::new();

		assert_eq!(leaf.insert(2, 2, UpdateOrInsert), None);
		assert_eq!(leaf.insert(1, 1, UpdateOrInsert), None);

		assert_eq!(leaf.insert(3, 33, FindOrInsert), None);
		assert_eq!(leaf.insert(4, 44, FindOrInsert), None);
		assert_eq!(leaf.insert(2, 22, UpdateOrInsert), Some(2));
		assert_eq!(leaf.insert(1, 11, UpdateOrInsert), Some(1));

		assert_eq!(leaf.insert(1, 111, FindOrInsert), Some(11));
		assert_eq!(leaf.insert(2, 222, FindOrInsert), Some(22));

		let values: Vec<_> = leaf.entries.iter().map(|(_, v)| *v).collect();
		assert_eq!(values, [11, 22, 33, 44])
	}

	#[test]
	fn to_from_bytes() {
		let mut leaf = Leaf::new();
		leaf.next = 1;
		leaf.prev = 2;
		leaf.entries.push((3, 3));
		leaf.entries.push((4, 4));

		let bytes = leaf.to_bytes();
		assert_eq!(bytes[0], 0); // Node type

		let leaf2 = Leaf::from_bytes(bytes);
		assert_eq!(leaf2.next, 1);
		assert_eq!(leaf2.prev, 2);
		assert_eq!(leaf.entries[..], leaf2.entries[..]);
	}

	#[test]
	fn split_at_mid() {
		let mut left = Leaf::new();
		left.entries = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)].to_vec();

		let (right, mid) = left.split_at_mid();
		assert_eq!(left.entries.len(), 2);
		assert_eq!(right.entries.len(), 3);
		assert_eq!(mid, 3);
	}
}