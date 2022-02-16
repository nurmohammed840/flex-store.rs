use std::convert::TryInto;

use bytes::{Buf, BufMut};

use crate::entry::Key;

#[derive(Debug, PartialEq)]
pub struct MetaInfo {
	key_size: u8,
	value_size: u8,
	block_size: u32,
}

impl MetaInfo {
	pub fn new<K: Key, V: Key, const BLOCK_SIZE: usize>() -> Self {
		Self {
			key_size: K::SIZE.try_into().unwrap(),
			value_size: V::SIZE.try_into().unwrap(),
			block_size: BLOCK_SIZE as u32,
		}
	}
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut v = Vec::new();
		v.put_u8(self.key_size);
		v.put_u8(self.value_size);
		v.put_u32_le(self.block_size);
		v
	}
	pub fn from_bytes(mut bytes: &[u8]) -> Self {
		Self {
			key_size: bytes.get_u8(),
			value_size: bytes.get_u8(),
			block_size: bytes.get_u32_le(),
		}
	}
}

pub struct Metadata {
	pub is_opened: u8,
	pub len: u32,
	pub root: u16,
}

impl Metadata {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut v = Vec::new();
		v.put_u8(self.is_opened);
		v.put_u32_le(self.len);
		v.put_u16_le(self.root);
		v
	}

	pub fn from_bytes(mut bytes: &[u8]) -> Self {
		Self {
			is_opened: bytes.get_u8(),
			len: bytes.get_u32_le(),
			root: bytes.get_u16_le(),
		}
	}
}
