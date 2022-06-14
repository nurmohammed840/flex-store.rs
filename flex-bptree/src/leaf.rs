use super::*;
use bin_layout::Record;
use std::mem;

#[derive(Debug, Clone)]
pub enum SetOpt {
    FindOrInsert,
    UpdateOrInsert,
}

#[derive(Encoder, Decoder)]
pub struct Leaf<K, V, const SIZE: usize> {
    pub next: u16,
    pub prev: u16,
    pub entries: Record<u16, Vec<(K, V)>>,
}

impl<K: Key, V: Key, const SIZE: usize> Leaf<K, V, SIZE> {
    pub fn capacity() -> usize {
        // BlockSize - (Node type (1) + next (2) + prev (2) + entries len (2))
        (SIZE - 7) / (K::SIZE + V::SIZE)
    }

    pub fn new() -> Self {
        Self {
            next: 0,
            prev: 0,
            entries: Record::new(Vec::with_capacity(Self::capacity())),
        }
    }

    pub fn is_full(&self) -> bool {
        self.entries.len() >= Self::capacity()
    }

    pub fn is_half_full(&self) -> bool {
        self.entries.len() > (Self::capacity() / 2)
    }

    pub fn binary_search(&self, key: &K) -> Result<usize, usize> {
        self.entries
            .binary_search_by(|(k, _)| k.partial_cmp(key).expect("Key can't be `NaN`"))
    }

    pub fn insert(&mut self, key: K, val: V, opt: SetOpt) -> Option<V> {
        match self.binary_search(&key) {
            Ok(index) => {
                let entry = self.entries.get_mut(index);
                match opt {
                    SetOpt::FindOrInsert => entry.map(|kv| kv.1),
                    SetOpt::UpdateOrInsert => entry.map(|kv| mem::replace(&mut kv.1, val)),
                }
            }
            Err(index) => {
                self.entries.insert(index, (key, val));
                None
            }
        }
    }

    pub fn split_at_mid(&mut self) -> Self {
        let mid_point = self.entries.len() / 2;
        Self {
            next: 0,
            prev: 0,
            entries: Record::new(self.entries.drain(mid_point..).collect()),
        }
    }
}
