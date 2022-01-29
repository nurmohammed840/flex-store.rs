mod branch;
mod entry;
mod leaf;
mod node;

use std::fs;
use std::io::Result;
use std::marker::PhantomData;

use async_recursion::async_recursion;
use flex_page::{PageNo, Pages};

use entry::Key;
use leaf::{Leaf, SetOption};
use node::Node;

pub struct BPlusTree<K, V, P: PageNo, const N: usize> {
    root: u32,
    meta_path: String,
    pages: Pages<P, N>,
    _marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Key, P: PageNo, const N: usize> BPlusTree<K, V, P, N> {
    pub async fn open(mut path: String) -> Result<Self> {
        path.push_str(".idx");
        let pages = Pages::open(&path)?;

        path.push_str(".meta");
        let root = if let Ok(bytes) = fs::read(&path) {
            u32::from_le_bytes(bytes.try_into().unwrap())
        } else {
            pages.alloc(1).await?
        };
        Ok(Self { root, pages, _marker: PhantomData, meta_path: path })
    }

    #[async_recursion]
    async fn find_leaf(&mut self, num: P, key: K) -> Result<Leaf<K, V, P, N>> {
        let buf = self.pages.read(num).await?;
        let node = Node::<K, V, P, N>::from_bytes(buf);
        match node {
            Node::Branch(branch) => {
                let index = branch.lookup(key);
                self.find_leaf(branch.childs[index], key).await
            }
            Node::Leaf(leaf) => Ok(leaf),
        }
    }

    pub async fn get(&mut self, key: K) -> Result<Option<V>> {
        Ok(self.find_leaf(P::new(self.root), key).await?.find(key))
    }

    #[async_recursion]
    async fn _set(&mut self, num: P, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
        let buf = self.pages.read(num).await?;
        let node = Node::<K, V, P, N>::from_bytes(buf);
        match node {
            Node::Branch(branch) => {
                let index = branch.lookup(key);
                let result = self._set(branch.childs[index], key, value, opt).await?;
                // let mut ret = None;
                // self._set(num, key, value, opt).await
                Ok(result)
            }
            Node::Leaf(mut leaf) => {
                // let mut ret = None;
                let result = leaf.insert(key, value, opt);
                if leaf.is_full() {
                    let (other, mid) = leaf.split_at_mid();
                    let new_num = self.pages.alloc(1).await?;
                }
                Ok(result)
            }
        }
    }
}

impl<K, V, P: PageNo, const N: usize> BPlusTree<K, V, P, N> {}

impl<K, V, P: PageNo, const N: usize> Drop for BPlusTree<K, V, P, N> {
    fn drop(&mut self) {
        fs::write(self.meta_path.clone(), self.root.to_le_bytes()).unwrap()
    }
}

// Entry Api
// pub fn clear(&mut self) -> Result<()> { todo!() }
// pub fn first_key_value(&self) -> Option<(&K, &V)>
// pub fn last_key_value(&self) -> Option<(&K, &V)>
// pub fn pop_first(&mut self) -> Option<(K, V)>
// pub fn pop_last(&mut self) -> Option<(K, V)>
// pub fn contains_key<Q>(&self, key: &Q) -> bool
// pub fn append(&mut self, other: &mut Self)
// pub fn remove(&mut self, key: &K) -> Option<V> { unimplemented!() }
// pub fn insert(&mut self, key: K, value: V) -> Option<V> { unimplemented!() }
