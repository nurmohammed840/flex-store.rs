use crate::entry::{Key, Value};
use bin_layout::{Decoder, Encoder, Record};

#[derive(Encoder, Decoder)]
pub struct Leaf<K, V, const SIZE: usize> {
    pub entries: Record<u16, Vec<(K, V)>>,
}

impl<K: Key, V: Value, const SIZE: usize> Leaf<K, V, SIZE> {
    pub const fn capacity() -> usize {
        // BlockSize - (Node type (1) + entries len (2))
        (SIZE - 3) / (K::SIZE + V::SIZE)
    }

    pub fn new() -> Self {
        Self {
            entries: Record::new(Vec::with_capacity(Self::capacity())),
        }
    }
}
