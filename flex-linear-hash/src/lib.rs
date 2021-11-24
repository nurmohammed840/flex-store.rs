#![allow(warnings)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use flex_page::Pages;

#[repr(packed)]
pub struct BucketIndex {
    pub round: u8,
    pub pointer: u32,
}

impl BucketIndex {
    pub fn get(&self, pages: &mut Pages<u16, 2, 4096>) -> Bucket {
        pages.read(self.pointer.into()).unwrap().into()
    }
    pub fn _to_bytes(self) -> [u8; 5] {
        unsafe { std::mem::transmute::<Self, [u8; 5]>(self) }
    }
    pub fn _from_bytes(buf: [u8; 5]) -> Self {
        unsafe { std::mem::transmute::<[u8; 5], Self>(buf) }
    }
}

#[repr(C)]
struct Entry {
    id: u64,
    value: [u8; 6],
}

pub struct Bucket {
    next: u32,
    entrys: Vec<Entry>,
}

impl From<[u8; 4096]> for Bucket {
    fn from(buf: [u8; 4096]) -> Self {
        todo!()
    }
}

impl Bucket {
    fn insert(&mut self, hash: u64, value: [u8; 6]) {}
}

// struct Map {
//     next: u16,
//     bucket_indexes: Vec<BucketIndex>,
// }
// impl Map {
//     fn to_bytes(&self) -> Vec<u8> {
//         // let mut bytes = vec![];
//         // bytes.append(&mut self.next.to_le_bytes().to_vec());
//         // for v in self.bucket_indexes.iter() {
//         //     bytes.append(&mut v.to_bytes().to_vec());
//         // }
//         // bytes
//     }
//     fn from_bytes(bytes: Vec<u8>) -> Self {
//         // let next = u16::from_le_bytes(bytes)
//         Self {
//             next: todo!(),
//             bucket_indexes: todo!(),
//         }
//     }
// }

pub struct LinearHash {
    _pages: Pages<u16, 2, 4096>,
    /// indecate next split pointer index in `bucket_indexes`;
    next: u32,
    bucket_indexes: Vec<BucketIndex>,
}

impl LinearHash {
    fn _open(filepath: &str) -> std::io::Result<Self> {
        let _pages: Pages<u16, 2, 4096> = Pages::open(filepath)?;

        todo!()
        // Ok(Self {
        //     pages,
        //     next: todo!(),
        //     bucket_indexes: todo!(),
        // })
    }

    pub fn insert<K: Hash>(&mut self, key: K, value: [u8; 6]) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let bucket_index =
            self.find_bucket_index(hash, self.bucket_indexes[self.next as usize].round);

        // let mut bucket = bucket_index.get(&mut self._pages);
        // bucket.insert(hash, value);
    }

    fn find_bucket_index(&self, hash: u64, round: u8) -> &BucketIndex {
        let index = hash % (2u64).pow(round.into());
        let bucket_index = &self.bucket_indexes[index as usize];
        if bucket_index.round != round {
            return self.find_bucket_index(hash, bucket_index.round);
        }
        bucket_index
    }
}
