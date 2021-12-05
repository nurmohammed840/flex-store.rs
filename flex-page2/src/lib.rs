#![allow(warnings)]

use std::{fs::File, io::*, marker::PhantomData};

use meta::Metadata;
use page_no::PageNo;

mod meta;
mod page;
mod page_no;

pub struct Pages<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> {
    file: File,
    metadata: Metadata<P, PS, PAGE_SIZE>,
    _marker: PhantomData<P>,
}

impl<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> Pages<P, PS, PAGE_SIZE> {
    fn open(path: &str) -> Result<Self> {
        let file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        if file.metadata()?.len() % PAGE_SIZE as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        // let mut this = Self {
        //     file,
        //     metadata: Metadata::new(PS, PAGE_SIZE),
        //     _marker: PhantomData,
        // };

        todo!()
    }
}

impl<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> Drop for Pages<P, PS, PAGE_SIZE> {
    fn drop(&mut self) {
        todo!()
    }
}
