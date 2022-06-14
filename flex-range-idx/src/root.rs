use crate::entry::Key;
use bin_layout::{ Array, Encoder, Decoder, Cursor};

pub struct Root<K, const SIZE: usize> {
    keys: Vec<K>,
    childs: Vec<u16>,
}

impl<K: Key, const SIZE: usize> Encoder for Root<K, SIZE> {
    fn encoder(self, buf: &mut impl Array<u8>) {
        (self.keys.len() as u16).encoder(buf);
        for key in self.keys {
            key.encoder(buf);
        }
        for child in self.childs {
            child.encoder(buf);
        }
    }
}

impl<'de, K: Key, const SIZE: usize> Decoder<'de, ()> for Root<K, SIZE> {
    fn decoder(c: &mut Cursor<&'de [u8]>) -> Result<Self, ()> {
        let keys_len = u16::decoder(c)?;
        let mut this = Self{
            keys: Vec::with_capacity(keys_len as usize),
            childs: Vec::with_capacity(keys_len as usize + 1),
        };
        for _ in 0..keys_len {
            this.keys.push(K::decoder(c)?);
        }
        for _ in 0..keys_len + 1 {
            this.childs.push(u16::decoder(c)?);
        }
        Ok(this)
    }
}
