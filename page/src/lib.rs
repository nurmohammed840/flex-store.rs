use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io::Result;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use bytes::{Buf, BufMut};
use flex_page::PageNo;

pub struct Pages<K: PageNo, const NBYTES: usize> {
    pages: flex_page::Pages<K, NBYTES>,
    path: PathBuf,
    free: RwLock<Vec<K>>,
    store: RwLock<HashMap<u64, Vec<u8>>>,
}

impl<K: PageNo, const NBYTES: usize> Pages<K, NBYTES> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut path = path.as_ref().to_owned();

        path.set_extension("db");
        let pages = flex_page::Pages::open(&path)?;

        let mut free = Vec::new();
        let mut store = HashMap::new();

        path.set_extension("meta");
        if let Ok(bytes) = fs::read(&path) {
            let mut view = bytes.as_slice();
            let free_len = view.get_u32_le();
            let store_len = view.get_u16_le();

            free.reserve(free_len as usize);
            store.reserve(store_len as usize);

            for _ in 0..free_len {
                free.push(K::from_bytes(&view.copy_to_bytes(K::SIZE)));
            }
            for _ in 0..store_len {
                let key = view.get_u64_le();
                let len = view.get_u16_le();
                let value = view.copy_to_bytes(len as usize).to_vec();
                store.insert(key, value);
            }
        }
        Ok(Self { pages, store: RwLock::new(store), free: RwLock::new(free), path })
    }

    pub fn set_metadata(&self, key: impl Hash, value: Vec<u8>) {
        self.store.write().unwrap().insert(create_hash(key), value);
    }

    pub fn get_metadata(&self, key: impl Hash) -> Option<Vec<u8>> {
        let data = self.store.read().unwrap().get(&create_hash(key))?.clone();
        Some(data)
    }

    pub fn remove_metadata(&self, key: impl Hash) -> Option<Vec<u8>> {
        self.store.write().unwrap().remove(&create_hash(key))
    }

    /// This function is synchronous.
    pub fn sync_metadata(&self) -> Result<()> {
        let mut bytes = Vec::new();
        let free = self.free.read().unwrap();
        let store = self.store.read().unwrap();

        bytes.put_u32_le(free.len() as u32);
        bytes.put_u16_le(store.len() as u16);

        for key in free.iter() {
            bytes.put(key.to_bytes().as_slice());
        }
        for (&key, value) in store.iter() {
            bytes.put_u64_le(key);
            bytes.put_u16_le(value.len() as u16);
            bytes.put(value.as_slice());
        }
        fs::write(self.path.as_path(), bytes)
    }
}

impl<K: PageNo, const NBYTES: usize> Deref for Pages<K, NBYTES> {
    type Target = flex_page::Pages<K, NBYTES>;
    fn deref(&self) -> &Self::Target {
        &self.pages
    }
}

impl<K: PageNo, const NBYTES: usize> Drop for Pages<K, NBYTES> {
    fn drop(&mut self) {
        self.sync_metadata().unwrap();
    }
}

fn create_hash(val: impl Hash) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let mut map = HashMap::new();
        map.insert("k", "v");
        map.insert("k", "v1");
        println!("{:#?}", map);
    }

    // #[test]
    // fn test_name() {
    //     rm_file();
    //     let _pages = Pages::<u16, 4096>::open("test").unwrap();
    //     // _pages.sync().unwrap();
    // }

    // fn rm_file() {
    //     let _ = fs::remove_file("test.db");
    //     let _ = fs::remove_file("test.meta");
    // }

}
