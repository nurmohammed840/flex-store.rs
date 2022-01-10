use crate::{page_no::PageNo, Pages};
use stack_array::Array;
use std::{
    fs::File,
    io::{Cursor, Result, Write},
    ops::{Deref, DerefMut},
    sync::Mutex,
};
use utils::cursor::{Reader, Writer};

struct Free<K, const NBYTES: usize>
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
        let mut this = Self {
            file,
            curr,
            prev: 0,
            list: Array::new(),
        };
        this.update_from(Pages::<K, NBYTES>::read_sync(file, curr)?);
        Ok(this)
    }

    pub fn to_buf(&self) -> [u8; NBYTES] {
        let mut buf = [0; NBYTES];
        let mut view = Cursor::new(&mut buf[..]);

        view.set(self.prev).unwrap();
        view.set(self.list.len() as u32).unwrap();

        for num in self.list.iter() {
            view.write(&num.to_bytes()).unwrap();
        }
        buf
    }

    pub fn update_from(&mut self, buf: [u8; NBYTES]) {
        let mut view = Cursor::new(buf);

        self.prev = view.get::<u32>().unwrap();
        let len = view.get::<u32>().unwrap() as usize;

        self.list.clear();
        for i in 0..len {
            self.list.push(K::from_bytes(view.buf().unwrap()));
        }
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

// impl<K, const NBYTES: usize> Drop for Free<K, NBYTES>
// where
//     K: PageNo,
//     [(); (NBYTES - 8) / K::SIZE]:,
// {
//     fn drop(&mut self) {
//         let list = self.list.lock().unwrap();
//         Pages::<K, NBYTES>::write_sync(self.file, self.curr, &list.to_buf()).unwrap();
//     }
// }