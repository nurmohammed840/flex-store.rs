mod branch;
mod view;
mod entry;
mod leaf;
mod meta;
mod node;

use flex_page::Pages;

use meta::{MetaInfo, Metadata};
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use std::path::Path;

use branch::Branch;
use entry::Key;
use leaf::Leaf;
use node::Node;

pub use view::View;
pub use leaf::SetOption;

pub struct BTree<K, V, const SIZE: usize> {
	len: u32,
	root: u16,
	pages: Pages<SIZE>,
	_marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Key, const SIZE: usize> BTree<K, V, SIZE> {
	/// #### _Blocking_
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		let file = File::options()
			.read(true)
			.write(true)
			.create(true)
			.open(path)?;

		let pages = Pages::open(file)?;
		let metainfo = MetaInfo::new::<K, V, SIZE>();

		let mut len = 0;
		let mut root = 1;
		let mut raw_meta = [0; SIZE];

		if pages.len() == 0 {
			pages.alloc(2)?; // 1 for metadata, 1 for root node
			raw_meta[..6].copy_from_slice(&metainfo.to_bytes());
		} else {
			raw_meta = pages.read(0)?;
			let info = MetaInfo::from_bytes(&raw_meta[..6]);
			if info != metainfo {
				return Err(Error::new(
					ErrorKind::InvalidInput,
					format!("Expected: {:?}, but got: {:?}", info, metainfo),
				));
			}
			let metadata = Metadata::from_bytes(&raw_meta[6..]);
			if metadata.is_opened == 1 {
				return Err(Error::new(
					ErrorKind::AddrInUse,
					"The file is already opened",
				));
			}
			len = metadata.len;
			root = metadata.root;
		};
		// `metadata.is_opened` flag.
		raw_meta[6] = 1;
		pages.write(0, raw_meta)?;
		Ok(Self {
			len,
			root,
			pages,
			_marker: PhantomData,
		})
	}

	pub fn len(&self) -> u32 {
		self.len
	}

	/// #### _Blocking_
	pub fn clear(&mut self) -> Result<()> {
		self.len = 0;
		self.root = 1;
		self.pages.write(self.root as u64, [0; SIZE])
	}

	/// #### _Blocking_
	pub fn compact(&self) {
		unimplemented!()
	}

	/// #### _Blocking_
	pub fn set(&mut self, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
		let result = self._set(self.root, key, value, opt)?;
		if let Some((mid, right)) = result.1 {
			let root_branch = Branch::create_root(mid, self.root, right);
			// TODO: reuse free pages
			self.root = self.pages.create(root_branch.to_bytes())? as u16;
		};
		if let None = result.0 {
			self.len += 1;
		}
		Ok(result.0)
	}

	/// #### _Blocking_
	pub fn head(&self) -> Result<View<K, V, SIZE>> {
		Ok(View {
			pages: &self.pages,
			leaf: self._head(self.root)?,
		})
	}

	/// #### _Blocking_
	pub fn tail(&self) -> Result<View<K, V, SIZE>> {
		Ok(View {
			pages: &self.pages,
			leaf: self._tail(self.root)?,
		})
	}

	/// #### _Blocking_
	pub fn get(&self, key: K) -> Result<View<K, V, SIZE>> {
		Ok(View {
			pages: &self.pages,
			leaf: self._get(self.root, key)?,
		})
	}
	fn _head(&self, num: u16) -> Result<Leaf<K, V, SIZE>> {
		match Node::from_bytes(self.pages.read(num as u64)?) {
			Node::Branch(b) => self._head(*b.childs.first().unwrap()),
			Node::Leaf(leaf) => Ok(leaf),
		}
	}
	fn _tail(&self, num: u16) -> Result<Leaf<K, V, SIZE>> {
		match Node::from_bytes(self.pages.read(num as u64)?) {
			Node::Branch(b) => self._tail(*b.childs.last().unwrap()),
			Node::Leaf(leaf) => Ok(leaf),
		}
	}
	fn _get(&self, num: u16, key: K) -> Result<Leaf<K, V, SIZE>> {
		match Node::from_bytes(self.pages.read(num as u64)?) {
			Node::Branch(b) => self._get(b.child_at(b.lookup(key)), key),
			Node::Leaf(leaf) => Ok(leaf),
		}
	}
	fn _set(
		&self,
		num: u16,
		key: K,
		value: V,
		opt: SetOption,
	) -> Result<(Option<V>, Option<(K, u16)>)> {
		let ret;
		let mut marge = None;

		match Node::from_bytes(self.pages.read(num as u64)?) {
			Node::Branch(mut branch) => {
				let index = branch.lookup(key);
				let (val, entry) = self._set(branch.child_at(index), key, value, opt)?;
				ret = val;
				if let Some(e) = entry {
					branch.insert(index, e);
					if branch.is_full() {
						let (other, mid) = branch.split_at_mid();
						// TODO: reuse free pages
						marge = Some((mid, self.pages.create(other.to_bytes())? as u16));
					}
					self.pages.write(num as u64, branch.to_bytes())?;
				}
			}
			Node::Leaf(mut leaf) => {
				ret = leaf.insert(key, value, opt.clone());
				// If `FindOrInsert` option is enable, And if the key is found, Return the value.
				// So we don't need to write the page.
				if matches!(opt, SetOption::FindOrInsert) && ret.is_some() {
					return Ok((ret, None));
				}
				// If the leaf is full, split it.
				if leaf.is_full() {
					let (mut right, mid) = leaf.split_at_mid();
					right.prev = num;
					// TODO: reuse free pages
					leaf.next = self.pages.create(right.to_bytes())? as u16;
					marge = Some((mid, leaf.next));
				}
				self.pages.write(num as u64, leaf.to_bytes())?;
			}
		}
		Ok((ret, marge))
	}
}

impl<K, V, const SIZE: usize> Drop for BTree<K, V, SIZE> {
	fn drop(&mut self) {
		let mut meta = self.pages.read(0).unwrap();
		let metadata = Metadata {
			is_opened: 0,
			len: self.len,
			root: self.root,
		};
		meta[6..13].copy_from_slice(&metadata.to_bytes());
		self.pages.write(0, meta).unwrap();
	}
}

// ============================================================================

#[cfg(test)]
mod debug_tree {
	use super::*;
	#[derive(Debug)]
	#[allow(dead_code)]
	enum Tree<K, V> {
		Branch {
			keys: Vec<K>,
			childs: Vec<Tree<K, V>>,
		},
		Leaf(Vec<(K, V)>),
	}
	impl<K: Key, V: Key, const SIZE: usize> BTree<K, V, SIZE> {
		fn build_tree(&self, num: u16) -> Tree<K, V> {
			// Create the runtime
			match Node::<K, V, SIZE>::from_bytes(self.pages.read(num as u64).unwrap()) {
				Node::Leaf(leaf) => Tree::Leaf(leaf.entries),
				Node::Branch(branch) => Tree::Branch {
					keys: branch.keys,
					childs: branch
						.childs
						.iter()
						.map(|&n| self.build_tree(n.into()))
						.collect(),
				},
			}
		}
		fn debug_tree(&self) -> Tree<K, V> {
			self.build_tree(self.root)
		}
	}

	#[test]
	#[ignore = "Only for debugging purpose"]
	fn debug_tree() -> Result<()> {
		let mut tree: BTree<u16, u16, 64> = BTree::open("debug_tree").unwrap();
		for i in 1..=1000 {
			tree.set(i, i, SetOption::UpdateOrInsert).unwrap();
		}
		std::fs::write("tree.txt", format!("{:#?}", tree.debug_tree()))?;
		std::fs::remove_file("debug_tree")
	}
}
