use crate::page_no::PageNo;
use crate::Pages;
use std::fs::File;
use std::io::Result;

use data_view::DataView;
use stack_array::Array;

pub struct Free<K, const NBYTES: usize>
where
    K: PageNo,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    file: &'static File,
    curr: u32,

    prev: u32,
    list: Array<K, { (NBYTES - 8) / K::SIZE }>,
}

impl<K, const NBYTES: usize> Free<K, NBYTES>
where
    K: PageNo,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    pub fn new(file: &'static File, curr: u32) -> Result<Self> {
        let mut this = Self { file, curr, prev: 0, list: Array::new() };
        this.read()?;
        Ok(this)
    }

    pub fn write(&self) -> Result<()> {
        let mut view = DataView::new([0; NBYTES]);

        view.write(self.prev);
        view.write(self.list.len() as u32);

        for num in self.list.iter() {
            view.write_slice(num.to_bytes());
        }
        Pages::<K, NBYTES>::_write(self.file, self.curr, view.data)
    }

    pub fn read(&mut self) -> Result<()> {
        let mut view = DataView::new(Pages::<K, NBYTES>::_read(self.file, self.curr)?);

        self.prev = view.read::<u32>();
        let len = view.read::<u32>() as usize;

        self.list.clear();
        for _ in 0..len {
            self.list.push(K::from_bytes(view.read_buf()));
        }
        Ok(())
    }

    pub async fn push(&mut self, num: K) -> Result<()> {
        if self.list.is_full() {
            // let (buf, _) = tokio::try_join!(
            //     Pages::<K, NBYTES>::read_async(self.file, list.prev),
            //     Pages::<K, NBYTES>::write_async(self.file, self.curr, list.to_buf())
            // )?;
            // list.update_from(buf)
        } else {
            self.list.push(num);
        }
        Ok(())
    }

    pub async fn pop(&mut self) -> Option<K> {
        loop {
            if !self.list.is_empty() {
                return Some(self.list.pop());
            }
            if self.prev == 0 {
                return None;
            }
            // self.update_from(Pages::<K, NBYTES>::read_async(self.file, self.prev).await.ok()?);
        }
    }
}

impl<K, const NBYTES: usize> Drop for Free<K, NBYTES>
where
    K: PageNo,
    [(); (NBYTES - 8) / K::SIZE]:,
{
    fn drop(&mut self) {
        self.write().unwrap();
    }
}
