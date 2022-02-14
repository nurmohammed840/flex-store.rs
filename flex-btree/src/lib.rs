mod branch;
mod entry;
mod leaf;
mod node;

use bytes::{Buf, BufMut};
use std::cell::Cell;
use std::fs::File;
use std::io::Result;
use std::marker::PhantomData;
use std::path::Path;

use async_recursion::async_recursion;
use flex_page::{PageNo, Pages};

use branch::Branch;
use entry::Key;
use leaf::Leaf;
use node::Node;

pub use leaf::SetOption;

macro_rules! find_leaf {
	[$data: expr, $body: expr] => {
		match Node::from_bytes($data) {
			Node::Branch(branch) => {
				let f = $body;
				f(branch).await
			}
			Node::Leaf(leaf) => Ok(leaf),
		}
	};
}

pub struct BTree<K, V, N: PageNo, const SIZE: usize> {
	len: Cell<u32>,
	root: Cell<N>,
	pages: Pages<N, SIZE>,
	_marker: PhantomData<(K, V)>,
}

unsafe impl<K, V, N: PageNo, const SIZE: usize> Send for BTree<K, V, N, SIZE> {}
unsafe impl<K, V, N: PageNo, const SIZE: usize> Sync for BTree<K, V, N, SIZE> {}

impl<K: Key, V: Key, N: PageNo, const SIZE: usize> BTree<K, V, N, SIZE> {
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		let file = File::options()
			.read(true)
			.write(true)
			.create(true)
			.open(path)?;

		let pages = Pages::open(file)?;

		let mut metadata = unsafe { &*pages.metadata() };
		let root = metadata.get_u32_le();
		let root = if root == 0 {
			pages.alloc_sync(1)?
		} else {
			N::new(root)
		};
		let len = metadata.get_u32_le().into();
		Ok(Self {
			len,
			pages,
			root: Cell::new(root),
			_marker: PhantomData,
		})
	}

	pub fn len(&self) -> u32 {
		self.len.get()
	}

	pub async fn set(&self, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
		let root = self.root.get();
		let _guard = self.pages.lock_api.write(root).await;

		let result = self._set(root, key, value, opt).await?;
		if let Some((mid, right)) = result.1 {
			let root_branch = Branch::create_root(mid, root, right);
			// TODO: reuse free pages
			let new_root = self.pages.create(root_branch.to_bytes()).await?;
			self.root.set(new_root);
		};
		if let None = result.0 {
			self.len.set(self.len.get() + 1);
		}
		Ok(result.0)
	}

	pub async fn clear(&self) -> Result<()> {
		let root = N::new(1);
		let _guard = self.pages.lock_api.write(root).await;

		self.root.set(root);
		self.pages.write(root, [0; SIZE]).await
	}

	pub fn compact(&self) {
		unimplemented!()
	}

	pub async fn get(&self, key: K) -> Result<Option<V>> {
		let root = self.root.get();
		let _guard = self.pages.lock_api.read(root).await;
		Ok(self._find_leaf(root, key).await?.find(key))
	}

	pub async fn first_key_value(&self) -> Result<Option<(K, V)>>  {
		let root = self.root.get();
		let _guard = self.pages.lock_api.read(root).await;
		Ok(self._first_leaf(root).await?.entries.first().copied())
	}

	pub async fn last_key_value(&self) -> Result<Option<(K, V)>>  {
		let root = self.root.get();
		let _guard = self.pages.lock_api.read(root).await;
		Ok(self._last_leaf(root).await?.entries.last().copied())
	}

	// get first leaf node, and return its first key
	#[async_recursion]
	async fn _first_leaf(&self, num: N) -> Result<Leaf<K, V, N, SIZE>> {
		find_leaf!(self.pages.get(num).await?, |b: Branch<K, N, SIZE>| {
			self._first_leaf(*b.childs.first().unwrap())
		})
	}
	#[async_recursion]
	async fn _last_leaf(&self, num: N) -> Result<Leaf<K, V, N, SIZE>> {
		find_leaf!(self.pages.get(num).await?, |b: Branch<K, N, SIZE>| {
			self._last_leaf(*b.childs.last().unwrap())
		})
	}
	#[async_recursion]
	async fn _find_leaf(&self, num: N, key: K) -> Result<Leaf<K, V, N, SIZE>> {
		find_leaf!(self.pages.get(num).await?, |b: Branch<K, N, SIZE>| {
			self._find_leaf(b.child_at(b.lookup(key)), key)
		})
	}

	#[async_recursion]
	async fn _set(
		&self,
		num: N,
		key: K,
		value: V,
		opt: SetOption,
	) -> Result<(Option<V>, Option<(K, N)>)> {
		let ret;
		let mut marge = None;

		match Node::from_bytes(self.pages.get(num).await?) {
			Node::Branch(mut branch) => {
				let index = branch.lookup(key);
				let (val, entry) = self._set(branch.child_at(index), key, value, opt).await?;
				ret = val;
				if let Some(e) = entry {
					branch.insert(index, e);
					if branch.is_full() {
						let (other, mid) = branch.split_at_mid();
						// TODO: reuse free pages
						marge = Some((mid, self.pages.create(other.to_bytes()).await?));
					}
					self.pages.write(num, branch.to_bytes()).await?;
				}
			}
			Node::Leaf(mut leaf) => {
				let find_opt = matches!(opt, SetOption::FindOrInsert);
				ret = leaf.insert(key, value, opt);
				// If `FindOrInsert` option is enable, And if the key is found, Return the value.
				// So we don't need to write the page.
				if find_opt && ret.is_some() {
					return Ok((ret, None));
				}
				// If the leaf is full, split it.
				if leaf.is_full() {
					let (mut right, mid) = leaf.split_at_mid();
					right.prev = num;
					// TODO: reuse free pages
					leaf.next = self.pages.create(right.to_bytes()).await?;
					marge = Some((mid, leaf.next));
				}
				self.pages.write(num, leaf.to_bytes()).await?;
			}
		}
		Ok((ret, marge))
	}
}

impl<K, V, N: PageNo, const SIZE: usize> Drop for BTree<K, V, N, SIZE> {
	fn drop(&mut self) {
		let mut metadata = unsafe { self.pages.metadata() };
		metadata.put_u32_le(self.root.get().as_u32());
		metadata.put_u32_le(self.len.get());
	}
}

// ============================================================================

#[cfg(test)]
mod format_tree {
	#![allow(warnings)]
	use super::Node::*;
	use super::*;
	type K = u64;
	type V = u8;
	type N = u8;
	type Node = super::Node<K, V, N, 64>;
	type BTree = super::BTree<K, V, N, 64>;
	#[derive(Debug)]
	enum Tree {
		Branch { keys: Vec<K>, childs: Vec<Tree> },
		Leaf(Vec<K>),
	}
	#[async_recursion]
	async fn build_tree(n: N, pages: &Pages<N, 64>) -> Tree {
		match Node::from_bytes(pages.get(n).await.unwrap()) {
			Branch(branch) => Tree::Branch {
				keys: branch.keys.to_vec(),
				childs: {
					let mut childs = Vec::with_capacity(branch.childs.len());
					for c in branch.childs {
						childs.push(build_tree(c, pages).await);
					}
					childs
				},
			},
			Leaf(leaf) => Tree::Leaf(leaf.entries.iter().map(|(k, _)| *k).collect()),
		}
	}
	#[tokio::test]
	#[ignore = "Only for debugging purpose"]
	async fn debug_tree() -> Result<()> {
		{
			let tree = BTree::open("debug_tree").unwrap();
			for i in 1..=500 {
				tree.set(i, 0, SetOption::UpdateOrInsert).await.unwrap();
			}
			let contents = format!("{:?}", build_tree(tree.root.get(), &tree.pages).await);
			std::fs::write("tree.txt", contents)?;
		}
		std::fs::remove_file("debug_tree")
	}
}
