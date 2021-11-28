use std::ops::{Deref, DerefMut};
use crate::{prelude::*, Value};

#[derive(Clone, Default, PartialEq)]
pub struct Array(Vec<Value>);

impl Deref for Array {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Array {
    #[inline]
    pub fn new() -> Self {
        Array::default()
    }

    #[inline]
    pub fn push<T: FlexVal>(&mut self, value: T) {
        self.0.push(value.to_flex_val())
    }

    #[inline]
    pub fn get(&self, index: usize) -> &Value {
        self.0.get(index).unwrap_or(&Value::Null)
    }

    #[inline]
    pub fn insert<T: FlexVal>(&mut self, index: usize, value: T) {
        self.0.insert(index, value.to_flex_val());
    }

    #[inline]
    pub fn fill<T: FlexVal>(&mut self, value: T) {
        self.0.fill(value.to_flex_val());
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Value {
        if self.len() >= index {
            return Value::Null;
        }
        self.0.remove(index)
    }
}
