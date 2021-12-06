use byte_seeker::{BytesReaderLE, BytesSeeker, BytesWriterLE};

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::page_no::PageNo;

pub struct Metadata<P, const PAGE_SIZE: usize> {
    size_info: SizeInfo,
    /// Free list page pointer
    free_list_page_no: P,
    data: HashMap<u64, Vec<u8>>,
}

impl<P: PageNo, const PAGE_SIZE: usize> Metadata<P, PAGE_SIZE> {
    pub fn new() -> Self {
        assert!(PAGE_SIZE >= 64, "Page size should >= 64 bytes");
        assert!(PAGE_SIZE < 16777216, "Page size should < 16mb");
        Self {
            size_info: SizeInfo {
                page_no_nbytes: P::NBYTES as u8,
                page_size: PAGE_SIZE as u32,
            },
            free_list_page_no: P::default(),
            data: HashMap::new(),
        }
    }
    /// This funtion return expected `SizeInfo` as err!
    pub(crate) fn extend_from_bytes(&mut self, bytes: &[u8]) -> Result<(), SizeInfo> {
        let mut reader = BytesSeeker::new(bytes);

        let size_info = SizeInfo::from(reader.buf::<4>());
        if size_info != self.size_info {
            return Err(size_info);
        }

        self.free_list_page_no = P::from_bytes(reader.bytes(P::NBYTES).to_vec());

        let data_len: u16 = reader.read();
        self.data = HashMap::with_capacity(data_len.into());

        for _ in 0..data_len {
            let key: u64 = reader.read();
            let vlen: u16 = reader.read();
            let value = reader.bytes(vlen.into()).to_vec();
            self.data.insert(key, value);
        }
        Ok(())
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.size_info.to_bytes());
        bytes.extend(self.free_list_page_no.to_bytes());

        bytes.write(self.data.len() as u16);
        for (key, value) in &self.data {
            bytes.write(*key);
            // Max Value Size: 64 kb
            let vlen = value.len() as u16;
            bytes.write(vlen);
            bytes.extend(&value[..vlen.into()]);
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
    fn to_bytes(&self) -> [u8; 4] {
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
        let mut m1 = Metadata::<u32, 8192>::new();
        let mut m2 = Metadata::<u32, 8192>::new();
        let mut m3 = Metadata::<u32, 8192>::new();

        m1.free_list_page_no = 123;
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
        assert_eq!(m3.size_info, m1.size_info);
        assert_eq!(m1.free_list_page_no, m3.free_list_page_no);
    }

    #[test]
    fn metadata_size_info() {
        let mut m1 = Metadata::<u16, 4096>::new();
        let mut m2 = Metadata::<u32, 8192>::new();
        assert_eq!(m1.to_bytes().len(), 8);
        assert_eq!(m2.to_bytes().len(), 10);
        assert_eq!(
            m2.extend_from_bytes(&m1.to_bytes()),
            Err(SizeInfo { 
                page_size: 4096,
                page_no_nbytes: 2
            })
        );
    }

    fn create_btree_map_from_hash_map(hashmap: &HashMap<u64, Vec<u8>>) -> BTreeMap<&u64, &Vec<u8>> {
        let mut btreemap = BTreeMap::new();
        for (k, v) in hashmap {
            btreemap.insert(k, v);
        }
        btreemap
    }
}
