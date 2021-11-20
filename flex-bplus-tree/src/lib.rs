#![allow(warnings)]
mod branch;
mod leaf;
mod node;
mod util;
mod entry;

use std::marker::PhantomData;

struct BPlusTree<T, const S: usize> {
    _marker: PhantomData<T>,
}

impl<T, const S: usize> Default for BPlusTree<T, S> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<T, const S: usize> BPlusTree<T, S> {
    fn set(&self, id: T, value: [u8; S]) {}
}
