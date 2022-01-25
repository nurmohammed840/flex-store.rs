#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod node;
mod leaf;
mod entry;
mod branch;

use std::marker::PhantomData;

use flex_page::{PageNo, Pages};

pub struct BPlusTree<K, V, P: PageNo = u16, const PAGE_SIZE: usize = 4096> {
    _root:  P,
    _pages: Pages<P, PAGE_SIZE>,
    _marker:   PhantomData<(K, V)>,
}

impl<K, V, P: PageNo, const PAGE_SIZE: usize> BPlusTree<K, V, P, PAGE_SIZE> {
    pub fn new(root: P, pages: Pages<P, PAGE_SIZE>) -> Self {
        Self { _root: root, _pages: pages, _marker: PhantomData }
    }

    // pub fn get(&self, key: &K) -> Option<&V> { unimplemented!() }
    // pub fn insert(&mut self, key: K, value: V) { unimplemented!() }
}




// ====================================================================================================

// mod branch;
// mod entry;
// mod leaf;
// mod node;

// use flex_page::{Pages, U24};
// use std::marker::PhantomData;

// pub use entry::Key;
// pub use flex_page::PageNo;

// struct BPlusTree<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
// {
//     pages: Pages<P, PS, PAGE_SIZE>,
//     root_page_no: P,
//     _key_marker: PhantomData<K>,
//     _value_marker: PhantomData<V>,
// }

// impl<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
//     BPlusTree<K, V, P, KS, VS, PS, PAGE_SIZE>
// where
//     K: Key<KS>,
//     V: Key<VS>,
//     P: PageNo<PS>,
// {
//     fn open(filepath: &str) -> Result<Self, std::io::Error> {
//         let mut pages = Pages::<P, PS, PAGE_SIZE>::open(filepath)?;

//         Ok(Self {
//             pages,
//             root_page_no: todo!(),
//             _key_marker: PhantomData,
//             _value_marker: PhantomData,
//         })
//     }
// }

// macro_rules! BPlusTree {
//     ($key:ty, $value:ty) => {
//         BPlusTree!($key, $value, 4096)
//     };
//     ($key:ty, $value:ty, $page_size:literal) => {
//         BPlusTree!($key, $value, $crate::U24, $page_size)
//     };
//     ($key:ty, $value:ty, $page_len:ty) => {
//         BPlusTree!($key, $value, $page_len, 4096)
//     };
//     ($key:ty, $value:ty, $page_len:ty, $page_size:literal) => {
//         BPlusTree::<
//             $key,
//             $value,
//             $page_len,
//             { <$key>::SIZE },
//             { <$value>::SIZE },
//             { <$page_len>::SIZE },
//             $page_size,
//         >
//     };
// }
