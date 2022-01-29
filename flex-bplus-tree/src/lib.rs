#![allow(dead_code)]

mod entry;
mod branch;
mod leaf;
mod node;

use std::io::Result;
// use std::marker::PhantomData;

// use async_recursion::async_recursion;
// use page::{PageNo, Pages};

// use entry::Key;
// use leaf::{Leaf, SetOption};
// use node::Node;

// pub struct BPlusTree<K, V, P: PageNo = u16, const PAGE_SIZE: usize = 4096> {
//     root:    P,
//     pages:   Pages<P, PAGE_SIZE>,
//     _marker: PhantomData<(K, V)>,
// }

// impl<K, V, P: 'static, const PAGE_SIZE: usize> BPlusTree<K, V, P, PAGE_SIZE>
// where
//     K: Key,
//     V: Key,
//     P: PageNo,
//     [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
//     [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
//     [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
// {
//     pub fn new(root: P, pages: Pages<P, PAGE_SIZE>) -> Self {
//         Self { root, pages, _marker: PhantomData }
//     }

//     #[async_recursion]
//     async fn find_leaf(&mut self, num: P, key: K) -> Result<Leaf<K, V, P, PAGE_SIZE>> {
//         let buf = self.pages.read(num).await?;
//         let node = Node::<K, V, P, PAGE_SIZE>::from_bytes(buf);
//         match node {
//             Node::Branch(branch) => {
//                 let index = branch.lookup(key);
//                 self.find_leaf(branch.childs[index], key).await
//             }
//             Node::Leaf(leaf) => Ok(leaf),
//         }
//     }

//     pub async fn get(&mut self, key: K) -> Result<Option<V>> {
//         Ok(self.find_leaf(self.root, key).await?.find(key))
//     }

//     // Entry Api
//     // pub fn clear(&mut self) -> Result<()> { todo!() }
//     // pub fn first_key_value(&self) -> Option<(&K, &V)>
//     // pub fn last_key_value(&self) -> Option<(&K, &V)>
//     // pub fn pop_first(&mut self) -> Option<(K, V)>
//     // pub fn pop_last(&mut self) -> Option<(K, V)>
//     // pub fn contains_key<Q>(&self, key: &Q) -> bool
//     // pub fn append(&mut self, other: &mut Self)
//     // pub fn remove(&mut self, key: &K) -> Option<V> { unimplemented!() }
//     // pub fn insert(&mut self, key: K, value: V) -> Option<V> { unimplemented!() }

//     #[async_recursion]
//     async fn _set(&mut self, num: P, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
//         let buf = self.pages.read(num).await?;
//         let node = Node::<K, V, P, PAGE_SIZE>::from_bytes(buf);
//         match node {
//             Node::Branch(branch) => {
//                 let index = branch.lookup(key);
//                 let result = self._set(branch.childs[index], key, value, opt).await?;
//                 // let mut ret = None;
//                 // self._set(num, key, value, opt).await
//                 Ok(result)
//             }
//             Node::Leaf(mut leaf) => {
//                 // let mut ret = None;
//                 let result = leaf.insert(key, value, opt);
//                 if leaf.entries.is_full() {
//                     let (other, mid) = leaf.split_at_mid();
//                     // let new_num = self.pages.alloc().await?;
//                     // let new_num = self.pages.alloc();
//                 }
//                 Ok(result)
//             }
//         }
//     }
// }

// // ====================================================================================================
// mod t {
//     use super::*;

//     use data_view::DataView;
//     use std::collections::{HashMap, HashSet};
//     use std::fs::File;
//     use std::io::{Read, Write};
//     use std::path::Path;

//     struct Meta {
//         file:  File,
//         store: HashMap<u64, Vec<u8>>,
//         free:  HashSet<u32>,
//     }

//     impl Meta {
//         pub fn open(path: impl AsRef<Path>) -> Result<Self> {
//             let mut file = File::options().read(true).create(true).write(true).open(path)?;
//             let mut store = HashMap::new();
//             let mut free = HashSet::new();

//             let mut buf = Vec::new();
//             file.read_to_end(&mut buf)?;
//             let mut view = DataView::new(buf.as_slice());

//             let store_len = view.read::<u16>();
//             let free_len = view.read::<u32>();

//             for _ in 0..store_len {
//                 let key = view.read::<u64>();
//                 let len = view.read::<u16>();
//                 let value = view.read_slice(len as usize).to_owned();
//                 store.insert(key, value);
//             }
//             for _ in 0..free_len {
//                 free.insert(view.read::<u32>());
//             }
//             Ok(Self { file, store, free })
//         }

//         pub fn set(&mut self, key: u64, value: impl AsRef<[u8]>) {
//             let data = value.as_ref();
//             assert!(data.len() < u16::MAX as usize, "value too large");
//             self.store.insert(key, data.to_vec());
//         }

//         pub fn get(&self, key: u64) -> Option<&Vec<u8>> { self.store.get(&key) }
//     }

//     impl Drop for Meta {
//         fn drop(&mut self) {
//             let mut bytes = Vec::new();

//             bytes.extend_from_slice(&(self.store.len() as u16).to_le_bytes());
//             bytes.extend_from_slice(&(self.free.len() as u32).to_le_bytes());

//             for (key, value) in self.store.iter() {
//                 bytes.extend_from_slice(&key.to_le_bytes());
//                 bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
//                 bytes.extend_from_slice(&value);
//             }
//             for num in self.free.iter() {
//                 bytes.extend_from_slice(&num.to_le_bytes());
//             }
//             self.file.write_all(bytes.as_slice()).unwrap();
//         }
//     }
// }
