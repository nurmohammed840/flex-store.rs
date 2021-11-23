#![allow(warnings)]
mod branch;
mod entry;
mod leaf;
mod node;
mod util;

use std::marker::PhantomData;

use flex_page::Pages;

struct BPlusTree<const PAGE_SIZE: usize> {
    pages: Pages<u16, PAGE_SIZE>,
    root_page_no: u16,
}

impl<const PAGE_SIZE: usize> BPlusTree<PAGE_SIZE> {
    fn open(filepath: &str) -> Self {
        todo!()
    }
}

struct BPlus<const P: usize>;

impl<const P: usize> BPlus<P> {
    fn new() -> Self {
        Self
    }
    fn page_size(self) -> Self {
        self
    }
}

fn s() {
    let f: BPlus<5> = BPlus::new();
}

// struct BPlusTree<T, const S: usize> {
//     _marker: PhantomData<T>,
// }

// impl<T, const S: usize> Default for BPlusTree<T, S> {
//     fn default() -> Self {
//         Self {
//             _marker: Default::default(),
//         }
//     }
// }

// impl<T, const S: usize> BPlusTree<T, S> {
//     fn set(&self, id: T, value: [u8; S]) {}
// }

// ------------------------------------------------------------------
