use byte_seeker::{BytesReader, LittleEndian};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::page_no::PageNo;

pub struct Metadata<P, const PS: usize, const PAGE_SIZE: usize> {
    /// Total Pages count
    len: P,
    size_info: SizeInfo,
    data: HashMap<u64, Vec<u8>>,
}

impl<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> Metadata<P, PS, PAGE_SIZE> {
    pub fn new() -> Self {
        assert!(PAGE_SIZE >= 64, "Page size should >= 64 bytes");
        assert!(PAGE_SIZE < 16777216, "Page size should < 16mb");
        Self {
            len: P::default(),
            data: HashMap::new(),
            size_info: SizeInfo {
                page_no_nbytes: PS as u8,
                page_size: PAGE_SIZE as u32,
            },
        }
    }

    /// This funtion return expected `SizeInfo` as err!
    pub(crate) fn extend_from_bytes(&mut self, bytes: &[u8]) -> Result<(), SizeInfo> {
        let mut reader = BytesReader::new(bytes);

        let size_info = SizeInfo::from(reader.read_buf::<4>());

        if size_info != self.size_info {
            return Err(size_info);
        }

        let data_len: u16 = reader.read();
        self.data = HashMap::with_capacity(data_len.into());

        for _ in 0..data_len {
            let key: u64 = reader.read();
            let vlen: u16 = reader.read();
            let value = reader.read_bytes(vlen.into()).to_vec();
            self.data.insert(key, value);
        }
        Ok(())
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(&self.size_info.to_buf());

        let map_len: u16 = self.data.len().try_into().unwrap();
        bytes.extend_from_slice(&map_len.to_le_bytes());

        for (k, v) in &self.data {
            bytes.extend_from_slice(&k.to_le_bytes());
            // value
            let vlen: u16 = v.len().try_into().unwrap();
            bytes.extend_from_slice(&vlen.to_le_bytes());
            bytes.extend_from_slice(&v);
        }
        bytes
    }

    pub fn insert<T: Hash>(&mut self, key: T, value: Vec<u8>) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.data.insert(hash, value);
    }
}

#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    page_size: u32,
    page_no_nbytes: u8,
}
impl SizeInfo {
    fn to_buf(&self) -> [u8; 4] {
        let [x, y, z, _] = self.page_size.to_le_bytes();
        [self.page_no_nbytes, x, y, z]
    }
    fn from(buf: [u8; 4]) -> Self {
        let [a, b, c, d] = buf;
        Self {
            page_no_nbytes: a,
            page_size: u32::from_le_bytes([b, c, d, 0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn metadata() {
        let mut m1 = Metadata::<u32, 4, 8192>::new();
        let mut m2 = Metadata::<u32, 4, 8192>::new();
        let mut m3 = Metadata::<u32, 4, 8192>::new();

        for i in 0..1000 {
            m1.insert(i, b"Nice!".to_vec());
        }
        m2.extend_from_bytes(&m1.to_bytes()).unwrap();
        m3.extend_from_bytes(&m2.to_bytes()).unwrap();
        // Rust, default hashmap doesn't maintain ordering...
        let a = create_btree_map_from_hash_map(&m1.data);
        let b = create_btree_map_from_hash_map(&m2.data);
        let c = create_btree_map_from_hash_map(&m3.data);

        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn metadata_size_info() {
        let mut m1 = Metadata::<u16, 2, 4096>::new();
        let mut m2 = Metadata::<u32, 4, 8192>::new();
        let expected_size_info = m2.extend_from_bytes(&m1.to_bytes()).err().unwrap();
        assert_eq!(
            expected_size_info,
            SizeInfo {
                page_size: 4096,
                page_no_nbytes: 2
            }
        )
    }

    #[test]
    fn size_info() {
        let size_info = SizeInfo {
            page_size: 4096,
            page_no_nbytes: 3,
        };
        assert!(size_info == SizeInfo::from(size_info.to_buf()));
    }

    fn create_btree_map_from_hash_map(hashmap: &HashMap<u64, Vec<u8>>) -> BTreeMap<&u64, &Vec<u8>> {
        let mut btreemap = BTreeMap::new();
        for (k, v) in hashmap {
            btreemap.insert(k, v);
        }
        btreemap
    }
}
