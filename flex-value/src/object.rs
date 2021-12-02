use std::ops::{Index, IndexMut};

use crate::{
    utils::{derives, extends},
    Value,
};

type Map<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Default, Clone, PartialEq, Debug, PartialOrd)]
pub struct Object(Map<String, Value>);

extends!(Object: Map<String, Value>);
derives!(Object: Display);
derives!(Object: New);

impl Object {
    #[inline]
    pub fn insert<V: Into<Value>>(&mut self, key: &str, value: V) -> Option<Value> {
        self.0.insert(key.to_string(), value.into())
    }

    pub fn to_string(&self) -> String {
        let mut string = "{".to_string();
        let mut iter = self.iter();
        if let Some((key, value)) = iter.next() {
            string.push_str(&format!("{:?}:{}", key, value.to_string()));
        }
        for (key, value) in iter {
            string.push_str(&format!(",{:?}:{}", key, value.to_string()));
        }
        string.push('}');
        string
    }
}

impl Index<&str> for Object {
    type Output = Value;
    #[inline]
    fn index(&self, key: &str) -> &Self::Output {
        self.0.get(key).unwrap()
    }
}
impl IndexMut<&str> for Object {
    #[inline]
    fn index_mut(&mut self, key: &str) -> & mut Self::Output {
        if !self.0.contains_key(key) {
            self.insert(key, Value::Null);
        }
        self.0.get_mut(key).unwrap()
    }
}