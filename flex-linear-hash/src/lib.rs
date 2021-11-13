mod bucket_index;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bucket_index::BucketIndex;
use flex_page::Pages;

// #[derive(Clone, Copy, Default)]
pub struct Entry {
    pub id: u64,
    pub value: [u8; 8],
}

pub struct Bucket {}

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
    pages: Pages<4096>,
    /// indecate next split pointer index in `bucket_indexes`;
    next: u16,
    bucket_indexes: Vec<BucketIndex>,
}

impl LinearHash {
    fn open(filepath: &str) -> std::io::Result<Self> {
        let pages: Pages<4096> = Pages::open(filepath)?;

        
        todo!()
        // Ok(Self {
        //     pages,
        //     next: todo!(),
        //     bucket_indexes: todo!(),
        // })
    }

    pub fn insert<K: Hash>(&self, key: K, value: [u8; 8]) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let id = hasher.finish();

        let _entry = Entry { id, value };

        let bucket = self.find_bucket(id, self.bucket_indexes[self.next as usize].round);
    }

    fn find_bucket(&self, hash: u64, round: u8) -> &BucketIndex {
        let index = hash % (2u64).pow(round.into());
        let bucket_index = &self.bucket_indexes[index as usize];
        if bucket_index.round != round {
            return self.find_bucket(hash, bucket_index.round);
        }
        bucket_index
    }
}
