#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod branch;
mod entry;
mod leaf;
mod node;

use std::io::Result;
use std::marker::PhantomData;

use flex_page::{PageNo, Pages};
use async_recursion::async_recursion;

use entry::Key;
use leaf::{Leaf, SetOption};
use node::Node;

pub struct BPlusTree<K, V, P: PageNo = u16, const PAGE_SIZE: usize = 4096> {
    root:    P,
    pages:   Pages<P, PAGE_SIZE>,
    _marker: PhantomData<(K, V)>,
}

impl<K, V, P: 'static, const PAGE_SIZE: usize> BPlusTree<K, V, P, PAGE_SIZE>
where
    K: Key,
    V: Key,
    P: PageNo,
    [(); (PAGE_SIZE - (1 + P::SIZE * 2 + 2)) / (K::SIZE + V::SIZE)]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE) - 1]:,
    [(); (PAGE_SIZE - (1 + 2)) / (K::SIZE + P::SIZE)]:,
{
    pub fn new(root: P, pages: Pages<P, PAGE_SIZE>) -> Self {
        Self { root, pages, _marker: PhantomData }
    }

    #[async_recursion]
    async fn find_leaf(&mut self, num: P, key: K) -> Result<Leaf<K, V, P, PAGE_SIZE>> {
        let buf = self.pages.read(num).await?;
        let node = Node::<K, V, P, PAGE_SIZE>::from_bytes(buf);
        match node {
            Node::Branch(branch) => {
                let index = branch.lookup(key);
                self.find_leaf(branch.childs[index], key).await
            }
            Node::Leaf(leaf) => Ok(leaf),
        }
    }

    pub async fn get(&mut self, key: K) -> Result<Option<V>> {
        Ok(self.find_leaf(self.root, key).await?.find(key))
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

    #[async_recursion]
    async fn _set(&mut self, num: P, key: K, value: V, opt: SetOption) -> Result<Option<V>> {
        let buf = self.pages.read(num).await?;
        let node = Node::<K, V, P, PAGE_SIZE>::from_bytes(buf);
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
                if leaf.entries.is_full() {
                    let (other, mid) = leaf.split_at_mid();
                    // let new_num = self.pages.alloc().await?;
                    // let new_num = self.pages.alloc();
                }
                Ok(result)
            }
        }
    }
}

// ====================================================================================================
