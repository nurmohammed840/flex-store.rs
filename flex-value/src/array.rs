use std::ops::{Index, IndexMut};

use crate::Value;

#[derive(Clone, Default, PartialEq, Debug, PartialOrd)]
pub struct Array(Vec<Value>);

crate::utils::extends!(Array: Vec<Value>);
crate::utils::derives!(Array: Display);
crate::utils::derives!(Array: New);

macro_rules! impl_method {
    [$name:ident $($arg:ident: $t:ty)*] => {
        #[inline]
        pub fn $name<T: Into<Value>>(&mut self, $($arg: $t,)* value: T) {
            self.0.$name($($arg,)* value.into());
        }
    };
}

impl Array {
    impl_method!(insert index: usize);
    impl_method!(resize new_len: usize);
    impl_method!(fill);
    impl_method!(push);

    pub fn to_string(&self) -> String {
        let mut string = "[".to_string();
        let mut iter = self.iter();
        if let Some(value) = iter.next() {
            string.push_str(&value.to_string());
        }
        for value in iter {
            string.push_str(&format!(",{}", value.to_string()));
        }
        string.push(']');
        string
    }
}

impl Index<usize> for Array {
    type Output = Value;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl IndexMut<usize> for Array {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if !self.0.get(index).is_some() {
            self.resize(index + 1, Value::Null);
        }
        self.0.get_mut(index).unwrap()
    }
}

impl<T: Into<Value>> FromIterator<T> for Array {
    #[inline]
    fn from_iter<R: IntoIterator<Item = T>>(iter: R) -> Self {
        let mut arr = Array::default();
        for v in iter {
            arr.push(v);
        }
        arr
    }
}
impl<T: Into<Value>, const S: usize> From<[T; S]> for Array {
    #[inline]
    fn from(arr: [T; S]) -> Self {
        arr.into_iter().collect()
    }
}
impl<T: Into<Value>> From<Vec<T>> for Array {
    #[inline]
    fn from(arr: Vec<T>) -> Self {
        arr.into_iter().collect()
    }
}