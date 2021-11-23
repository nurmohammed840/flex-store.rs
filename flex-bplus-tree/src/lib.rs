#![allow(warnings)]
mod branch;
mod entry;
mod leaf;
mod node;
mod util;

// use flex_page::Pages;

// use std::marker::PhantomData;

// struct BPlusTree<const PAGE_SIZE: usize> {
//     pages: Pages<u16, PAGE_SIZE>,
//     root_page_no: u16,
// }

// impl<const PAGE_SIZE: usize> BPlusTree<PAGE_SIZE> {
//     fn open(filepath: &str) -> Self {
//         todo!()
//     }
// }


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
//     fn set(&self, key: T, value: [u8; S]) {}
// }

// ------------------------------------------------------------------
