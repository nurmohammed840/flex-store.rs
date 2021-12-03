use crate::Value;

pub type Map = std::collections::HashMap<String, Value>;

pub trait Object {
    fn set<K: ToString, V: Into<Value>>(&mut self, key: K, value: V) -> Option<Value>;
    fn to_string(&self) -> String;
}

impl Object for Map {
    #[inline]
    fn set<K: ToString, V: Into<Value>>(&mut self, key: K, value: V) -> Option<Value> {
        self.insert(key.to_string(), value.into())
    }
    fn to_string(&self) -> String {
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