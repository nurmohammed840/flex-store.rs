use crate::{page_no::PageNo, Pages};
use std::{
    fs::File,
    io::{Cursor, Result, Write},
    sync::Mutex,
};
use utils::cursor::Reader;

pub struct Free<K, const NBYTES: usize>
where
    K: PageNo,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - 8) / K::SIZE)]:,
{
    file: &'static File,
    list: Mutex<List<K, NBYTES>>,
    pub(crate) curr_page_no: u32,
}

impl<K, const NBYTES: usize> Free<K, NBYTES>
where
    K: PageNo,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - 8) / K::SIZE)]:,
{
    pub fn new(curr_page_no: u32, file: &'static File) -> Result<Self> {
        let mut list = List::new(0);
        list.update_from(Pages::<K, NBYTES>::read_sync(file, curr_page_no)?);
        Ok(Self {
            file,
            curr_page_no,
            list: Mutex::new(list),
        })
    }

    async fn add(&self, num: K) -> Result<()> {
        let mut list = self.list.lock().unwrap();
        if list.push(num).is_none() {
            Pages::<K, NBYTES>::write_async(self.file, self.curr_page_no, list.to_buf()).await?;

            // let mut page = Pages::<K, NBYTES>::read_async(self.curr_page_no).await?;
            // page.update_from(list);

            // page.write_sync(self.file).await?;
            // list.clear();
            // list.push(num);
        }
        Ok(())

        // list.push(K::new(self.curr_page_no));
        // Pages::<K, NBYTES>::write_sync(self.file, self.curr_page_no, list.to_byte()).await;
    }

    async fn remove(&self) -> Option<K> {
        let mut list = self.list.lock().unwrap();
        loop {
            if let Some(num) = list.pop() {
                return Some(num);
            }
            let prev = list.prev.as_u32();
            if prev == 0 {
                return None;
            }
            list.update_from(Pages::<K, NBYTES>::read_async(self.file, prev).await.ok()?);
        }
    }
}

impl<K, const NBYTES: usize> Drop for Free<K, NBYTES>
where
    K: PageNo,
    [(); NBYTES - 8]:,
    [(); ((NBYTES - 8) / K::SIZE)]:,
{
    fn drop(&mut self) {
        let list = self.list.lock().unwrap().to_buf();
        Pages::<K, NBYTES>::write_sync(self.file, self.curr_page_no, &list).unwrap();
    }
}

pub struct List<K, const NBYTES: usize>
where
    K: PageNo,
    [(); ((NBYTES - 8) / K::SIZE)]:,
{
    prev: u32,
    len: u32,
    list: [K; ((NBYTES - 8) / K::SIZE)],
}

impl<K, const NBYTES: usize> List<K, NBYTES>
where
    K: PageNo,
    [(); ((NBYTES - 8) / K::SIZE)]:,
{
    pub fn new(prev: u32) -> Self {
        Self {
            prev,
            len: 0,
            list: [K::new(0); ((NBYTES - 8) / K::SIZE)],
        }
    }

    pub fn last(&self) -> Option<&K>  {
        self.list.get(self.len() - 1)
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn push(&mut self, num: K) -> Option<K> {
        if self.len() >= self.list.len() {
            return None;
        }
        self.list[self.len()] = num;
        self.len += 1;
        Some(num)
    }

    pub fn pop(&mut self) -> Option<K> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.list[self.len as usize])
        }
    }

    pub fn to_buf(&self) -> [u8; NBYTES] {
        let mut buf = [0; NBYTES];
        let mut view = Cursor::new(&mut buf[..]);
        view.write(&self.prev.to_le_bytes()).unwrap();
        view.write(&self.len.to_le_bytes()).unwrap();
        for i in 0..self.len() {
            view.write(&self.list[i].to_bytes()).unwrap();
        }
        buf
    }

    pub fn update_from(&mut self, buf: [u8; NBYTES]) {
        let mut view = Cursor::new(buf);
        self.prev = u32::from_le_bytes(view.buf().unwrap());
        self.len = u32::from_le_bytes(view.buf().unwrap());
        for i in 0..self.len() {
            self.list[i] = K::from_bytes(view.buf().unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut free_list = List::<u16, 12>::new(0);
        assert_eq!(free_list.list.len(), 2);
        assert_eq!(free_list.pop(), None);
        // assert_eq!(free_list.last(), None);
        assert_eq!(free_list.push(1), Some(1));
        assert_eq!(free_list.push(2), Some(2));
        assert_eq!(free_list.push(3), None);
        // assert_eq!(free_list.last(), Some(&2));
        assert_eq!(free_list.len(), 2);

        let buf = free_list.to_buf();
        let mut free_list = List::<u16, 12>::new(0);
        free_list.update_from(buf);

        assert_eq!(free_list.pop(), Some(2));
        assert_eq!(free_list.pop(), Some(1));
        assert_eq!(free_list.pop(), None);
        assert_eq!(free_list.len(), 0);
    }
}
