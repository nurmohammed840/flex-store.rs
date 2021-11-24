#![allow(warnings)]
mod branch;
mod entry;
mod leaf;
mod node;
mod util;

use flex_page::{Pages, U24};
use std::marker::PhantomData;

pub use entry::Key;
pub use flex_page::PageNo;

struct BPlusTree<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
{
    pages: Pages<P, PS, PAGE_SIZE>,
    root_page_no: P,
    _key_marker: PhantomData<K>,
    _value_marker: PhantomData<V>,
}

impl<K, V, P, const KS: usize, const VS: usize, const PS: usize, const PAGE_SIZE: usize>
    BPlusTree<K, V, P, KS, VS, PS, PAGE_SIZE>
where
    K: Key<KS>,
    V: Key<VS>,
    P: PageNo<PS>,
{
    fn open(filepath: &str) -> Self {
        Self {
            pages: Pages::<P, PS, PAGE_SIZE>::open(filepath).unwrap(),
            root_page_no: todo!(),
            _key_marker: PhantomData,
            _value_marker: PhantomData,
        }
    }
}

macro_rules! BPlusTree {
    ($key:ty, $value:ty) => {
        BPlusTree!($key, $value, 4096)
    };
    ($key:ty, $value:ty, $page_size:literal) => {
        BPlusTree!($key, $value, $crate::U24, $page_size)
    };
    ($key:ty, $value:ty, $page_len:ty) => {
        BPlusTree!($key, $value, $page_len, 4096)
    };
    ($key:ty, $value:ty, $page_len:ty, $page_size:literal) => {
        BPlusTree::<
            $key,
            $value,
            $page_len,
            { <$key>::SIZE },
            { <$value>::SIZE },
            { <$page_len>::SIZE },
            $page_size,
        >
    };
}

// ------------------------------------------------------------------
