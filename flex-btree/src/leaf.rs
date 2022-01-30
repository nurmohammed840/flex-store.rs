use std::mem::replace;

use bytes::{Buf, BufMut};
use flex_page::PageNo;

use crate::entry::Key;
use SetOption::*;

pub enum SetOption {
	UpdateOrInsert,
	FindOrInsert,
}

pub struct Leaf<K, V, N, const SIZE: usize> {
	pub next: N,
	pub prev: N,
	pub entries: Vec<(K, V)>,
}

impl<K: Key, V: Key, P: PageNo, const SIZE: usize> Leaf<K, V, P, SIZE> {
	pub fn capacity() -> usize {
		(SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)
	}
	pub fn new() -> Self {
		Self { next: P::new(0), prev: P::new(0), entries: Vec::with_capacity(Self::capacity()) }
	}

	pub fn is_full(&self) -> bool {
		self.entries.len() >= Self::capacity()
	}

	fn binary_search_by(&self, key: &K) -> Result<usize, usize> {
		self.entries.binary_search_by(|(k, _)| k.partial_cmp(key).expect("Key can't be `NaN`"))
	}

	pub fn insert(&mut self, key: K, value: V, opt: SetOption) -> Option<V> {
		match self.binary_search_by(&key) {
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

	pub fn find(&self, key: K) -> Option<V> {
		let index = self.binary_search_by(&key).ok()?;
		Some(self.entries.get(index)?.1)
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
		view.put(&self.next.to_bytes()[..]);
		view.put(&self.prev.to_bytes()[..]);
		view.put_u16_le(self.entries.len() as u16);

		for (key, value) in self.entries.iter() {
			view.put(&key.to_bytes()[..]);
			view.put(&value.to_bytes()[..]);
		}
		buf
	}

	pub fn from_bytes(mut bytes: &[u8]) -> Self {
		let mut this = Self::new();

		this.next = P::from_bytes(&bytes.copy_to_bytes(P::SIZE));
		this.prev = P::from_bytes(&bytes.copy_to_bytes(P::SIZE));
		let len = bytes.get_u16_le();

		for _ in 0..len {
			let key = K::from_bytes(&bytes.copy_to_bytes(K::SIZE));
			let value = V::from_bytes(&bytes.copy_to_bytes(V::SIZE));
			this.entries.push((key, value));
		}
		this
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	type Leaf<const N: usize> = super::Leaf<u8, u16, u8, 4096>;

	#[test]
	fn check_capacity() {
		assert_eq!(super::Leaf::<u64, u16, u16, 4096>::capacity(), 408);
		assert_eq!(super::Leaf::<u32, u16, flex_page::U24, 4096>::capacity(), 681);
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

		assert_eq!(leaf.find(3), Some(33));
		assert_eq!(leaf.find(4), Some(44));
		assert_eq!(leaf.find(99), None);

		let values = leaf.entries.iter().map(|(_, v)| *v).collect::<Vec<_>>();
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
		let mut view = &bytes[..];
		assert_eq!(view.get_u8(), 0); // Node type

		let leaf2 = Leaf::from_bytes(view);
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
