use std::mem::transmute;
use crate::{branch::Branch, leaf::Leaf};

#[repr(C, u64)]
pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn to_bytes(self) -> [u8; 4096] {
        unsafe { transmute::<Self, [u8; 4096]>(self) }
    }
    pub fn from_bytes(bytes: [u8; 4096]) -> Self {
        unsafe { transmute::<[u8; 4096], Self>(bytes) }
    }
}
