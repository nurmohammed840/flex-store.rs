use byte_seeker::{BytesReader, LittleEndian};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

pub struct Metadata {
    /// Total page number
    len: u32,
    size_info: SizeInfo,
    map: HashMap<u64, Vec<u8>>,
}

impl Metadata {
    pub fn new(page_no_nbytes: usize, page_size: usize) -> Self {
        assert!(page_size >= 64, "Page size should >= 64 bytes");
        assert!(page_size < 16777216, "Page size should < 16mb");
        Self {
            len: 0,
            map: HashMap::new(),
            size_info: SizeInfo {
                page_no_nbytes: page_no_nbytes as u8,
                page_size: page_size as u32,
            },
        }
    }
    /// This funtion return expected `SizeInfo` as err!
    pub(crate) fn extend_from_bytes(&mut self, bytes: &[u8]) -> Result<(), SizeInfo> {
        let mut reader = BytesReader::new(bytes);

        let size_info = SizeInfo::from(reader.buf::<4>());

        if size_info != self.size_info {
            return Err(size_info);
        }
        let map_len: u16 = reader.read();
        self.map = HashMap::with_capacity(map_len.into());

        for _ in 0..map_len {
            let key: u64 = reader.read();
            let vlen: u16 = reader.read();
            let value = reader.bytes(vlen.into()).to_vec();
            self.map.insert(key, value);
        }
        Ok(())
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.size_info.to_buf());

        let map_len: u16 = self.map.len().try_into().unwrap();
        bytes.extend_from_slice(&map_len.to_le_bytes());

        for (k, v) in &self.map {
            bytes.extend_from_slice(&k.to_le_bytes());
            // value
            let vlen: u16 = v.len().try_into().unwrap();
            bytes.extend_from_slice(&vlen.to_le_bytes());
            bytes.extend_from_slice(&v);
        }
        bytes
    }

    pub fn insert<T: Hash>(&mut self, key: T, value: Vec<u8>) {
        self.map.insert(create_hash(key), value);
    }
    pub fn get<T: Hash>(&self, key: T) -> Option<&Vec<u8>> {
        self.map.get(&create_hash(key))
    }
    pub fn get_mut<T: Hash>(&mut self, key: T) -> Option<&mut Vec<u8>> {
        self.map.get_mut(&create_hash(key))
    }
}

fn create_hash<T: Hash>(key: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
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

    #[test]
    fn metadata() {
        let mut m1 = Metadata::new(4, 8192);
        let mut m2 = Metadata::new(4, 8192);
        let mut m3 = Metadata::new(4, 8192);

        for i in 0..1000 {
            m1.insert(i, b"Nice!".to_vec());
        }
        m2.extend_from_bytes(&m1.to_bytes()).unwrap();
        m3.extend_from_bytes(&m2.to_bytes()).unwrap();
        // Rust, default hashmap doesn't maintain ordering...
        let a = create_btree_map_from_hash_map(&m1.map);
        let b = create_btree_map_from_hash_map(&m2.map);
        let c = create_btree_map_from_hash_map(&m3.map);

        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn metadata_size_info() {
        let mut m1 = Metadata::new(2, 4096);
        let mut m2 = Metadata::new(4, 8192);
        let expected_size_info = m2.extend_from_bytes(&m1.to_bytes()).err().unwrap();
        assert_eq!(
            expected_size_info,
            SizeInfo {
                page_size: 4096,
                page_no_nbytes: 2
            }
        )
    }
}
