use crate::{prelude::*, Value};
use std::collections::{btree_map, BTreeMap};

#[derive(Default, Debug, Clone)]
pub struct Object(BTreeMap<String, Value>);

impl Object {
    #[inline]
    pub fn new() -> Self {
        Object::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    #[inline]
    pub fn iter(&self) -> btree_map::Iter<String, Value> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> btree_map::IterMut<String, Value> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    #[inline]
    pub fn insert<V: FlexVal>(&mut self, key: &str, value: V) -> Option<Value> {
        self.0.insert(key.to_string(), value.to_flex_val())
    }

    #[inline]
    pub fn keys(&self) -> btree_map::Keys<String, Value> {
        self.0.keys()
    }

    #[inline]
    pub fn values(&self) -> btree_map::Values<String, Value> {
        self.0.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> btree_map::ValuesMut<String, Value> {
        self.0.values_mut()
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn to_string(&self) {
        todo!()
    }

    #[inline]
    pub fn own(&self) -> &BTreeMap<String, Value> {
        &self.0
    }

    #[inline]
    pub fn own_mut(&mut self) -> &mut BTreeMap<String, Value> {
        &mut self.0
    }
}

#[test]
fn test_name() {}
