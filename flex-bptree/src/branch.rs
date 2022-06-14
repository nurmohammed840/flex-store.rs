use super::*;
use bin_layout::{Array, Cursor};
use std::mem;

pub struct Branch<K, const SIZE: usize> {
    pub keys: Vec<K>,
    pub childs: Vec<u16>,
}

impl<K: Key, const SIZE: usize> Branch<K, SIZE> {
    pub fn capacity() -> usize {
        // BlockSize - (Node type (1) + keys len (2))
        (SIZE - 3) / (K::SIZE + mem::size_of::<u16>())
    }

    pub fn new() -> Self {
        Self {
            keys: Vec::with_capacity(Self::capacity() - 1),
            childs: Vec::with_capacity(Self::capacity()),
        }
    }

    pub fn is_full(&self) -> bool {
        self.childs.len() >= Self::capacity()
    }
}

impl<K: Key, const SIZE: usize> Encoder for Branch<K, SIZE> {
    fn encoder(self, buf: &mut impl Array<u8>) {
        (self.keys.len() as u16).encoder(buf);
        self.keys.into_iter().for_each(|k| k.encoder(buf));
        self.childs.into_iter().for_each(|c| c.encoder(buf));
    }
}

impl<'de, K: Key, const SIZE: usize> Decoder<'de, ()> for Branch<K, SIZE> {
    fn decoder(c: &mut Cursor<&'de [u8]>) -> Result<Self, ()> {
        let keys_len = u16::decoder(c)?;
        let mut this = Self::new();
        for _ in 0..keys_len {
            this.keys.push(K::decoder(c)?);
        }
        for _ in 0..keys_len + 1 {
            this.childs.push(u16::decoder(c)?);
        }
        Ok(this)
    }
}
