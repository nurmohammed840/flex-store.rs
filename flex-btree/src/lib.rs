#![allow(warnings)]

mod branch;
mod entry;
mod leaf;
mod meta;
mod node;
mod view;

use flex_page::Pages;

use meta::{MetaInfo, Metadata};
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use std::path::Path;

use branch::{Branch, DeleteOperation};
use entry::Key;
use leaf::Leaf;
use node::Node;

pub use leaf::SetOption;
pub use view::View;

pub enum Get<K> {
	First,
	Last,
	Exact(K),
}

pub struct BPlusTree<K, V, const SIZE: usize> {
	len: u32,
	root: u16,
	pages: Pages<SIZE>,
	_marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Key, const SIZE: usize> BPlusTree<K, V, SIZE> {
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
		self.pages.write(self.root as u64, [0; SIZE])?;
		self.pages.set_len(2)
	}

	/// #### _Blocking_
	pub fn compact(&self) {
		unimplemented!()
	}

	/// #### _Blocking_
	pub fn get(&self, opt: Get<K>) -> Result<View<K, V, SIZE>> {
		let mut page_no = self.root;
		let leaf = loop {
			page_no = match Node::from_bytes(self.pages.read(page_no as u64)?) {
				Node::Branch(b) => match opt {
					Get::First => b.childs[0],
					Get::Last => *b.childs.last().unwrap(),
					Get::Exact(key) => b.child_at(b.lookup(&key)),
				},
				Node::Leaf(leaf) => break leaf,
			}
		};
		Ok(View {
			leaf,
			pages: &self.pages,
		})
	}

	pub fn delete(&mut self, key: &K) -> Result<Option<(K, V)>> {
		let bytes = self.pages.read(self.root as u64)?;
		let (page_no, is_key_present_in_idx, sibings) = match Node::from_bytes(bytes) {
			Node::Branch(branch) => {
				let lookup_idx = branch.lookup(key);
				(
					branch.child_at(lookup_idx),
					branch.keys.contains(&key),
					branch.sibings_at(lookup_idx),
				)
			}
			Node::Leaf(mut leaf) => {
				return match leaf.binary_search(&key) {
					Ok(index) => Ok(Some(leaf.entries.remove(index))),
					Err(_) => Ok(None),
				}
			}
		};
		todo!()
		// let result = self._delete(key, self.root, false)?;
		// if result.is_some() {
		// 	self.len -= 1;
		// }
		// Ok(result)
	}

	/// ### Delete Operation
	///
	/// The complexity of the delete procedure in the B+ Tree surpasses that of the insert and search functionality.
	/// Before going through the steps below, one must know these facts about a B+ tree of degree m.
	//
	/// 1. A node can have a maximum of m children. (i.e. 3)
	/// 2. A node can contain a maximum of m - 1 keys. (i.e. 2)
	/// 3. A node should have a minimum of ⌈m/2⌉ children. (i.e. 2)
	/// 4. A node (except root node) should contain a minimum of ⌈m/2⌉ - 1 keys. (i.e. 1)
	///
	/// ### Case 1:
	///
	/// The key to be deleted is present only at the leaf node not in the indexes (or internal nodes).
	/// There are two cases for it:
	///
	/// - There is more than the minimum number of keys in the node. Simply delete the key.
	///   > Deleting `40` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-1-b+tree.png" width=400/>
	///
	/// - There is an exact minimum number of keys in the node. Delete the key and borrow a key from the immediate sibling. Add the median key of the sibling node to the parent.
	///   > Deleting `5` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-2-b+tree.png" width=400/>
	///
	/// ### Case 2:
	///
	/// The key to be deleted is present in the internal nodes as well. Then we have to remove them from the internal nodes as well. There are the following cases for this situation.
	///
	/// - If there is more than the minimum number of keys in the node, simply delete the key from the leaf node and delete the key from the internal node as well.
	///   Fill the empty space in the internal node with the inorder successor.
	///   > Deleting `45` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-3-b+tree_0.png" width=350/>
	///
	/// - If there is an exact minimum number of keys in the node, then delete the key and borrow a key from its immediate sibling (through the parent).
	///   Fill the empty space created in the index (internal node) with the borrowed key.
	///   > Deleting `35` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-4-b+tree.png" width=350/>
	///
	/// - This case is similar to Case II(1) but here, empty space is generated above the immediate parent node.
	///   After deleting the key, merge the empty space with its sibling.
	///   Fill the empty space in the grandparent node with the inorder successor.
	///   > Deleting `25` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-5-b+tree.png" width=350/>
	///
	/// ### Case 3:
	///
	/// In this case, the height of the tree gets shrinked. It is a little complicated.Deleting 55 from the tree below leads to this condition. It can be understood in the illustrations below.
	/// > Deleting `55` <img src="https://cdn.programiz.com/sites/tutorial2program/files/deletion-6-b+tree.png" width=350/>
	fn _delete(
		&self,
		key: K,
		page_no: u16,
		key_is_idx: bool,
		(left_sibling, right_sibling): (Option<u16>, Option<u16>),
	) -> Result<(Option<(K, V)>, DeleteOperation<K>)> {
		use DeleteOperation::*;
		let mut deleted_entry = None;
		let mut operation = Nothing;

		match Node::from_bytes(self.pages.read(page_no as u64)?) {
			Node::Branch(mut branch) => {
				let lookup_idx = branch.lookup(&key);
				let res = self._delete(
					key,
					branch.child_at(lookup_idx),
					key_is_idx || branch.binary_search(&key).is_ok(),
					branch.sibings_at(lookup_idx),
				)?;
				deleted_entry = res.0;

				match res.1 {
					UpdateIdx(k) => {
						if key_is_idx {
							operation = UpdateIdx(key);
						} else {
							*branch.get_key_at(lookup_idx) = k;
						}
					}
					ReplaceIdx(k) => {
						let key = branch.get_key_at(lookup_idx);
						*key = k;
						if key_is_idx {
							operation = UpdateIdx(*key);
						}
					}
					RemoveIdx => {
						let _free_key = branch.remove_key_at(lookup_idx);
						let _free_page = branch.childs.remove(lookup_idx);
					}
					_ => {}
				}
			}
			Node::Leaf(mut leaf) => {
				if let Ok(idx) = leaf.binary_search(&key) {
					let should_steal_key_from_sibling = !leaf.is_half_full();
					deleted_entry = Some(leaf.entries.remove(idx));

					if should_steal_key_from_sibling {
						operation = loop {
							if let Some(link) = left_sibling {
								let mut sibling = Leaf::from_bytes(self.pages.read(link as u64)?);
								if sibling.is_half_full() {
									leaf.entries.push(sibling.entries.pop().unwrap());
									break ReplaceIdx(leaf.entries.last().unwrap().0);
								} else {
									// Margeing the sibling with the leaf.
									sibling.entries.extend(leaf.entries.drain(..));
								}
							}
							if let Some(link) = right_sibling {
								let mut sibling = Leaf::from_bytes(self.pages.read(link as u64)?);
								if sibling.is_half_full() {
									leaf.entries.insert(0, sibling.entries.remove(0));
									break ReplaceIdx(leaf.entries.first().unwrap().0);
								} else {
									sibling.entries.extend(leaf.entries.drain(..));
								}
							}
							break RemoveIdx;
						};
					} else if key_is_idx {
						operation = UpdateIdx(leaf.entries.first().unwrap().0);
					}
				}
			}
		}
		Ok((deleted_entry, operation))
	}

	/// #### _Blocking_
	pub fn set(&mut self, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
		let (ret, marge) = self._set(self.root, key, value, opt)?;
		if let Some((mid, right)) = marge {
			let root_branch = Branch::create_root(mid, self.root, right);
			// TODO: reuse free pages
			self.root = self.pages.create(root_branch.to_bytes())? as u16;
		};
		if ret.is_none() {
			self.len += 1;
		}
		Ok(ret)
	}

	fn _set(
		&self,
		num: u16,
		key: K,
		value: V,
		opt: SetOption,
	) -> Result<(Option<V>, Option<(K, u16)>)> {
		let val;
		let mut marge = None;

		match Node::from_bytes(self.pages.read(num as u64)?) {
			Node::Branch(mut branch) => {
				let index = branch.lookup(&key);
				let ret = self._set(branch.child_at(index), key, value, opt)?;
				val = ret.0;
				if let Some(e) = ret.1 {
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
				val = leaf.insert(key, value, opt.clone());
				// If `FindOrInsert` option is enable, And if the key is founded, Return early.
				// So we don't need to write the page.
				if matches!(opt, SetOption::FindOrInsert) && val.is_some() {
					return Ok((val, None));
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
		Ok((val, marge))
	}
}

impl<K, V, const SIZE: usize> Drop for BPlusTree<K, V, SIZE> {
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
	impl<K: Key, V: Key, const SIZE: usize> BPlusTree<K, V, SIZE> {
		fn build_tree(&self, num: u16) -> Tree<K, V> {
			match Node::from_bytes(self.pages.read(num as u64).unwrap()) {
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
		let mut tree: BPlusTree<u32, u32, 64> = BPlusTree::open("debug_tree").unwrap();
		for i in 1..=100 {
			tree.set(i, i, SetOption::UpdateOrInsert).unwrap();
		}
		std::fs::write("tree.txt", format!("{:#?}", tree.debug_tree()))?;
		std::fs::remove_file("debug_tree")
	}
}

// 1) Start at the root and go up to leaf node containing the key K
// 2) Find the node n on the path from the root to the leaf node containing K
//     A. If n is root, remove K
//          a. if root has more than one key, done
//          b. if root has only K
//             i) if any of its child nodes can lend a node
//                Borrow key from the child and adjust child links
//             ii) Otherwise merge the children nodes. It will be a new root
//          c. If n is an internal node, remove K
//             i) If n has at least ceil(m/2) keys, done!
//             ii) If n has less than ceil(m/2) keys,
//                 If a sibling can lend a key,
//                 Borrow key from the sibling and adjust keys in n and the parent node
//                     Adjust child links
//                 Else
//                     Merge n with its sibling
//                     Adjust child links
//          d. If n is a leaf node, remove K
//             i) If n has at least ceil(M/2) elements, done!
//                 In case the smallest key is deleted, push up the next key
//             ii) If n has less than ceil(m/2) elements
//             If the sibling can lend a key
//                 Borrow key from a sibling and adjust keys in n and its parent node
//             Else
//                 Merge n and its sibling
//                 Adjust keys in the parent node
