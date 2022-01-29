#![doc = include_str!("../README.md")]

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Result;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

use bytes::{Buf, BufMut};
pub use flex_page::*;

pub struct Pages<K: PageNo, const NBYTES: usize> {
    pages: flex_page::Pages<K, NBYTES>,
    path: PathBuf,
    freelist: Mutex<HashSet<K>>,
    store: RwLock<HashMap<u64, Vec<u8>>>,
}

impl<K: PageNo, const NBYTES: usize> Pages<K, NBYTES> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut path = path.as_ref().to_owned();

        path.set_extension("db");
        let pages = flex_page::Pages::open(&path)?;

        let mut freelist = HashSet::new();
        let mut store = HashMap::new();

        path.set_extension("meta");
        if let Ok(bytes) = fs::read(&path) {
            let mut view = bytes.as_slice();
            let free_len = view.get_u32_le();
            let store_len = view.get_u16_le();

            freelist.reserve(free_len as usize);
            store.reserve(store_len as usize);

            for _ in 0..free_len {
                freelist.insert(K::from_bytes(&view.copy_to_bytes(K::SIZE)));
            }
            for _ in 0..store_len {
                let key = view.get_u64_le();
                let len = view.get_u16_le();
                let value = view.copy_to_bytes(len as usize).to_vec();
                store.insert(key, value);
            }
        }
        Ok(Self { pages, store: RwLock::new(store), freelist: Mutex::new(freelist), path })
    }

    pub fn find_free_slot(&self) -> Option<K> {
        let mut freelist = self.freelist.lock().unwrap();
        let &num = freelist.iter().next()?;
        freelist.remove(&num);
        Some(num)
    }

    pub async fn find_or_alloc_free_slot(&self) -> Result<K> {
        match self.find_free_slot() {
            Some(slot) => Ok(slot),
            None => Ok(K::new(self.pages.alloc(1).await?)),
        }
    }

    pub fn free(&self, no: K) -> bool {
        assert!((1..self.pages.len()).contains(&no.as_u32()));
        self.freelist.lock().unwrap().insert(no)
    }

    pub fn freelist_len(&self) -> usize {
        self.freelist.lock().unwrap().len()
    }

    pub fn set_metadata(&self, key: impl Hash, value: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.store.write().unwrap().insert(create_hash(key), value.as_ref().to_vec())
    }

    pub fn get_metadata(&self, key: impl Hash) -> Option<Vec<u8>> {
        self.store.read().unwrap().get(&create_hash(key)).cloned()
    }

    pub fn remove_metadata(&self, key: impl Hash) -> Option<Vec<u8>> {
        self.store.write().unwrap().remove(&create_hash(key))
    }

    /// This function is synchronous.
    pub fn sync_metadata(&self) -> Result<()> {
        let mut bytes = Vec::new();
        let free = self.freelist.lock().unwrap();
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
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}
