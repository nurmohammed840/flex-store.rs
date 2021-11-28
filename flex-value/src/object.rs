use crate::{prelude::*, Value};
use std::collections::BTreeMap;

#[derive(Default, Clone, PartialEq)]
pub struct Object(BTreeMap<String, Value>);

impl std::ops::Deref for Object {
    type Target = BTreeMap<String, Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Object {
    #[inline]
    pub fn new() -> Self {
        Object::default()
    }

    #[inline]
    pub fn remove(&mut self, key: &str) -> Value {
        self.0.remove(key).unwrap_or(Value::Null)
    }

    #[inline]
    pub fn get(&self, key: &str) -> &Value {
        self.0.get(key).unwrap_or(&Value::Null)
    }

    #[inline]
    pub fn insert<V: FlexVal>(&mut self, key: &str, value: V) -> Value {
        self.0
            .insert(key.to_string(), value.to_flex_val())
            .unwrap_or(Value::Null)
    }
}