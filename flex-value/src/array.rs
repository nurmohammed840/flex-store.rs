use crate::{prelude::*, Value};
use std::slice;

#[derive(Clone, Default, PartialEq)]
pub struct Array(Vec<Value>);

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
    pub fn first(&mut self) -> &Value {
        self.0.first().unwrap_or(&Value::Null)
    }

    #[inline]
    pub fn last(&mut self) -> &Value {
        self.0.last().unwrap_or(&Value::Null)
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        self.0.pop().unwrap_or(Value::Null)
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
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.0.get_mut(index)
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

    #[inline]
    pub fn iter(&self) -> slice::Iter<Value> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<Value> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn own(&self) -> &Vec<Value> {
        &self.0
    }

    #[inline]
    pub fn own_mut(&mut self) -> &mut Vec<Value> {
        &mut self.0
    }

}
