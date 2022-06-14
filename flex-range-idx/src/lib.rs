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

fn get_buf<P, const SIZE: usize>(pages: &Pages<SIZE>, mut no: P) -> io::Result<Vec<u8>>
where
    P: Into<u64> + Copy + for<'de> Decoder<'de, io::Error>,
{
    let mut data = Vec::new();
    loop {
        let buf = pages.read(no.into())?;
        let mut cursor = Cursor::new(buf.as_ref());
        let mut num = P::decoder(&mut cursor)?;

        if num.into() == 0 {
            return Ok(data);
        } else {
            data.extend(cursor.remaining_slice());
            no = num;
        }
    }
}

// ----------------------------------------------------------------------

trait PageNo: Into<u64> + Default + Copy + for<'de> Decoder<'de, io::Error> {}
impl PageNo for u16 {}
impl PageNo for u32 {}

pub struct PageManager<const SIZE: usize> {
    pub pages: Pages<SIZE>,
    free: u32,
}

impl<const SIZE: usize> PageManager<SIZE> {
    fn new(path: impl AsRef<Path>) -> io::Result<(Self)> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let pages = Pages::open(file)?;
        let buf = get_buf(&pages, 0u16)?;

        Ok((Self { pages, free: 1 }))
    }
}

// impl<const SIZE: usize> Drop for PageManager<SIZE> {
//     fn drop(&mut self) {
//     }
// }