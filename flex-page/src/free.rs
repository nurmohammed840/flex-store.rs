use crate::page_no::PageNo;
use std::io::{Cursor, Write};
use utils::cursor::Reader;

struct Free<K, const NBYTES: usize>
where
    K: PageNo,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    last: u32,
    list: FreeList<K, NBYTES>,

}










pub struct FreeList<K, const NBYTES: usize>
where
    K: PageNo,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    next: K,
    prev: K,
    len: u32,
    list: [K; ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)],
}

impl<K, const NBYTES: usize> FreeList<K, NBYTES>
where
    K: PageNo,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    pub fn new() -> Self {
        Self {
            next: K::new(0),
            prev: K::new(0),
            len: 0,
            list: [K::new(0); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)],
        }
    }

    pub fn capacity(&self) -> u32 {
        self.list.len() as u32
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn push(&mut self, page: K) -> Option<K> {
        let len = self.len;
        if len >= self.capacity() {
            return None;
        }
        self.list[len as usize] = page;
        self.len += 1;
        Some(page)
    }

    pub fn pop(&mut self) -> Option<K> {
        let len = self.len;
        if len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.list[self.len as usize])
        }
    }

    pub fn to_byte(&self) -> [u8; NBYTES] {
        let mut buf = [0; NBYTES];
        let mut view = Cursor::new(&mut buf[..]);
        view.write(&self.next.to_bytes()).unwrap();
        view.write(&self.prev.to_bytes()).unwrap();
        view.write(&self.len.to_le_bytes()).unwrap();
        for i in 0..self.list.len() {
            view.write(&self.list[i].to_bytes()).unwrap();
        }
        buf
    }

    pub fn from(buf: [u8; NBYTES]) -> Self {
        let mut view = Cursor::new(buf);
        let next = K::from_bytes(view.buf().unwrap());
        let prev = K::from_bytes(view.buf().unwrap());
        let len = u32::from_le_bytes(view.buf().unwrap());

        let mut list = [K::new(0); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)];
        for i in 0..list.len() {
            list[i] = K::from_bytes(view.buf().unwrap());
        }
        Self {
            next,
            prev,
            len,
            list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut free_list = FreeList::<u16, 12>::new();
        assert_eq!(free_list.capacity(), 2);
        assert_eq!(free_list.pop(), None);
        assert_eq!(free_list.push(1), Some(1));
        assert_eq!(free_list.push(2), Some(2));

        assert_eq!(free_list.push(3), None);
        assert_eq!(free_list.len(), 2);

        let buf = free_list.to_byte();
        let mut free_list = FreeList::<u16, 12>::from(buf);

        assert_eq!(free_list.pop(), Some(2));
        assert_eq!(free_list.pop(), Some(1));
        assert_eq!(free_list.pop(), None);
        assert_eq!(free_list.len(), 0);
    }
}
