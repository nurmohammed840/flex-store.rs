use crate::{branch::Branch, leaf::Leaf};

pub enum Node<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> {
    #[allow(dead_code)]
    Leaf(Leaf<K, V, X, Y, PAGE_SIZE>),
    Branch(Branch<K, X, PAGE_SIZE>),
}

impl<K, V, const X: usize, const Y: usize, const PAGE_SIZE: usize> Node<K, V, X, Y, PAGE_SIZE> {}
