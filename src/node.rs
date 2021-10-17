use std::mem;

use crate::{branch::Branch, leaf::Leaf};

pub enum Node {
    Leaf(Leaf),
    Branch(Branch),
}

impl Node {
    pub fn to_bytes(self) -> [u8; 4096] {
        unsafe { mem::transmute::<Self, [u8; 4096]>(self) }
    }
    pub fn from_bytes(bytes: [u8; 4096]) -> Self {
        unsafe { mem::transmute::<[u8; 4096], Self>(bytes) }
    }
}
