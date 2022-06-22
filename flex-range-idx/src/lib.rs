#![allow(warnings)]

mod buffer;
mod entry;
mod leaf;
mod root;

use bin_layout::{Cursor, Decoder};
use flex_page::Pages;
use leaf::Leaf;
use root::Root;
use std::{fs::File, io, path::Path};

enum Node<K, V, const SIZE: usize> {
    Leaf(Leaf<K, V, SIZE>),
    Root(Root<K, SIZE>),
}

struct RangeIdx<K, V, const SIZE: usize> {
    pages: Pages<SIZE>,
    root: Node<K, V, SIZE>,
}

impl<K, V, const SIZE: usize> RangeIdx<K, V, SIZE> {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let pages = Pages::open(file)?;
        let buf = get_buf(&pages, 0u16)?;

        Ok(Self {
            pages,
            root: todo!(),
        })
    }
}


// ----------------------------------------------------------------------