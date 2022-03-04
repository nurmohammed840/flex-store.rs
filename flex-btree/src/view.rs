use super::*;
use std::{fmt, ops::Deref};

pub struct View<'a, K, V, const SIZE: usize> {
	pub(super) pages: &'a Pages<SIZE>,
	pub(super) leaf: Leaf<K, V, SIZE>,
}

impl<K: Key, V: Key, const SIZE: usize> View<'_, K, V, SIZE> {
	/// #### _Blocking_
	pub fn next(&mut self) -> Result<bool> {
		self._fetch(self.leaf.next as u64)
	}

	/// #### _Blocking_
	pub fn prev(&mut self) -> Result<bool> {
		self._fetch(self.leaf.prev as u64)
	}

	pub fn find_idx(&self, key: &K) -> Option<usize> {
		self.leaf.binary_search(key).ok()
	}

	pub fn find(&self, key: &K) -> Option<&(K, V)> {
		self.leaf.entries.get(self.find_idx(key)?)
	}

	fn _fetch(&mut self, num: u64) -> Result<bool> {
		if num == 0 {
			return Ok(false);
		}
		self.leaf = Leaf::from_bytes(self.pages.read(num)?);
		Ok(true)
	}
}

impl<K, V, const SIZE: usize> Deref for View<'_, K, V, SIZE> {
	type Target = Vec<(K, V)>;
	fn deref(&self) -> &Self::Target {
		&self.leaf.entries
	}
}
impl<K: fmt::Debug, V: fmt::Debug, const SIZE: usize> fmt::Debug for View<'_, K, V, SIZE> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(&self.leaf.entries).finish()
	}
}
