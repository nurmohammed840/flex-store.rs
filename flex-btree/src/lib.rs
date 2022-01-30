mod branch;
mod entry;
mod leaf;
mod node;

use std::io::Result;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::{fs, sync::atomic::AtomicU32};

use async_recursion::async_recursion;
use flex_page::{PageNo, Pages};

use entry::Key;
use leaf::Leaf;
use node::Node;

pub use leaf::SetOption;

use crate::branch::Branch;

pub struct BTree<K, V, N, const SIZE: usize> {
	root: AtomicU32,
	meta_path: String,
	pages: Pages<N, SIZE>,
	_marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Key, N: PageNo, const SIZE: usize> BTree<K, V, N, SIZE> {
	pub fn open(path: impl AsRef<str>) -> Result<Self> {
		let mut path = path.as_ref().to_string();
		path.push_str(".idx");
		let pages = Pages::<N, SIZE>::open(&path)?;

		path.push_str(".meta");
		let root = if let Ok(bytes) = fs::read(&path) {
			u32::from_le_bytes(bytes.try_into().unwrap())
		} else {
			pages.alloc_sync(1)?.as_u32()
		};
		Ok(Self { root: AtomicU32::new(root), pages, _marker: PhantomData, meta_path: path })
	}

	fn get_root(&self) -> N {
		N::new(self.root.load(Ordering::SeqCst))
	}

	fn set_root(&self, num: N) {
		self.root.store(num.as_u32(), Ordering::SeqCst);
	}

	#[async_recursion]
	async fn find_leaf(&self, num: N, key: K) -> Result<Leaf<K, V, N, SIZE>> {
		let buf = self.pages.read(num).await?;
		let node = Node::<K, V, N, SIZE>::from_bytes(buf);
		match node {
			Node::Branch(branch) => {
				let index = branch.lookup(key);
				self.find_leaf(branch.childs[index], key).await
			}
			Node::Leaf(leaf) => Ok(leaf),
		}
	}

	pub async fn get(&self, key: K) -> Result<Option<V>> {
		Ok(self.find_leaf(self.get_root(), key).await?.find(key))
	}

	pub async fn set(&self, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
		let root = self.get_root();
		let result = self._set(root, key, value, opt).await?;

		if let Some((key, right_page_no)) = result.1 {
			let root_branch = Branch::<K, N, SIZE>::create_root(key, root, right_page_no);
			let buf = Node::<K, V, N, SIZE>::Branch(root_branch).to_bytes();
			let new_root = self.pages.create(buf).await?; // TODO: reuse free pages
			self.set_root(new_root);
		};
		Ok(result.0)
	}

	#[async_recursion]
	async fn _set(
		&self,
		num: N,
		key: K,
		value: V,
		opt: SetOption,
	) -> Result<(Option<V>, Option<(K, N)>)> {
		let mut page = self.pages.goto(num).await?;
		let mut node = Node::<K, V, N, SIZE>::from_bytes(page.buf);
		match node {
			Node::Branch(ref mut branch) => {
				let index = branch.lookup(key);
				let mut result = self._set(branch.childs[index], key, value, opt).await?;
				if let Some(value) = result.1 {
					branch.insert(index, value);
					if branch.is_full() {
						let (other, mid) = branch.split_at_mid();
						let buf = Node::<K, V, N, SIZE>::Branch(other).to_bytes();
						let new_num = self.pages.create(buf).await?; // TODO: reuse free pages
						result.1 = Some((mid, new_num));
					}
					page.buf = node.to_bytes();
					page.write().await?;
				}
				Ok(result)
			}
			Node::Leaf(ref mut leaf) => {
				let mut marge = None;
				let val = leaf.insert(key, value, opt);
				if leaf.is_full() {
					let (mut other, mid) = leaf.split_at_mid();
					let new_num = self.pages.alloc(1).await?; // TODO: reuse free pages

					leaf.next = new_num;
					other.prev = num;

					self.pages.write(new_num, Node::Leaf(other).to_bytes()).await?;
					marge = Some((mid, new_num));
				}
				page.buf = node.to_bytes();
				page.write().await?;
				Ok((val, marge))
			}
		}
	}
}

impl<K, V, N, const SIZE: usize> Drop for BTree<K, V, N, SIZE> {
	fn drop(&mut self) {
		let root = self.root.load(Ordering::Relaxed);
		fs::write(self.meta_path.clone(), root.to_le_bytes()).unwrap()
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
		match Node::from_bytes(pages.read(n).await.unwrap()) {
			Branch(branch) => {
				let mut childs = Vec::with_capacity(branch.childs.len());
				for c in branch.childs {
					childs.push(build_tree(c, pages).await);
				}
				Tree::Branch { keys: branch.keys, childs }
			}
			Leaf(leaf) => Tree::Leaf(leaf.entries.iter().map(|(k, _)| *k).collect()),
		}
	}

	#[tokio::test]
	async fn debug_tree() {
		{
			let mut tree = BTree::open("debug_tree").unwrap();
			for i in 1..=50 {
				tree.set(i, 0, SetOption::UpdateOrInsert).await.unwrap();
			}
			std::fs::write(
				"tree.txt",
				format!("{:#?}", build_tree(tree.get_root(), &tree.pages).await),
			);
		}
		std::fs::remove_file("debug_tree.idx").unwrap();
		std::fs::remove_file("debug_tree.idx.meta").unwrap();
	}
}
