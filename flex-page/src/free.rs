use bytes::DataView;

use crate::page_no::PageNo;

pub struct FreeList<K, const NBYTES: usize>
where
    K: PageNo,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    next: K,
    prev: K,
    len: u32,
    list: Vec<K>,
}

impl<K, const NBYTES: usize> FreeList<K, NBYTES>
where
    K: PageNo,
    [(); ((NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE)]:,
{
    fn len(&self) -> u32 {
        self.len
    }

    fn push(&mut self, page: K) {
        let len = self.len;
        // panic, if overflow list
        self.list[len as usize] = page;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<K> {
        let len = self.len;
        if len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.list[len as usize])
        }
    }

    // fn to_byte(&self) -> [u8; NBYTES] {
    //     let mut buf = [0; NBYTES];
    //     buf.set_bytes(0, &self.next.to_bytes());
    //     buf.set_bytes(K::SIZE, &self.prev.to_bytes());
    //     buf.set::<u32>(self.len);
    //     let padding = (2 * K::SIZE) + 4;
    //     for i in 0..self.len as usize {
    //         buf.set_bytes(padding + (i * K::SIZE), &self.list[i].to_bytes());
    //     }
    //     buf
    // }

    // fn from_byte(&self, buf: [u8;NBYTES]) -> Self  {
    //     let next = buf.get_bytes(0, K::SIZE);
    //     let prev = buf.get_bytes(K::SIZE, K::SIZE);
    //     let len = buf.get::<u32>();
    //     let padding = (2 * K::SIZE) + 4;

    //     let mut list = [K::new(0); (NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE];

    //     // let mut list = [K::new(0); (NBYTES - ((2 * K::SIZE) + 4)) / K::SIZE];
    //     // let padding = (2 * K::SIZE) + 4;
    //     // for i in 0..self.len as usize {
    //     //     list[i] = K::from_bytes(buf.get_bytes(padding + (i * K::SIZE)));
    //     // }
    //     Self {   }
    // }
}