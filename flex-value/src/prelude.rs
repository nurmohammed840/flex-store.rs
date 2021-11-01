use std::{fmt::Display, ops::Index};

use crate::{Array, Object, Value};

pub trait FlexVal: IntoFlexVal {
    fn to_flex_val(&self) -> Value;
}

pub trait IntoFlexVal: Clone {
    fn to_flex_val(self) -> Value;
}

// ----------------------------- Macro -----------------------------

#[macro_export]
macro_rules! arr {
    [$($value:expr),* $(,)?] => ({
        let mut arr = $crate::Array::new();
        $(arr.push($value);)*
        arr
    });
}

#[macro_export]
macro_rules! obj {
    [$($key:expr => $value:expr),* $(,)?] => ({
        let mut map = $crate::Object::new();
        $(map.insert($key, $value);)*
        map
    });
}

// ----------------------------- Index -----------------------------

impl Index<usize> for Array {
    type Output = Value;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl Index<&str> for Object {
    type Output = Value;
    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
    }
}

// ----------------------------- Display Json String -----------------------------

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Boolean(boolean) => format!("{}", boolean),
            Value::Number(num) => format!("{}", num),
            Value::String(string) => format!("{:?}", string),
            Value::Array(arr) => arr.to_string(),
            Value::Object(obj) => obj.to_string(),
        }
    }
}

impl Array {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push('[');
        let mut iter = self.iter();
        if let Some(value) = iter.next() {
            string.push_str(&value.to_string());
        }
        for value in iter {
            string.push_str(&format!(", {}", value.to_string()));
        }
        string.push(']');
        string
    }
}

impl Object {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push('{');
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

macro_rules! impl_trait {
    ($name:ident for $($t:ty)*) => ($(
        impl $name for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.to_string())
            }
        }
    )*)
}
impl_trait!(Display for Value Array Object);

// ----------------------------- All-Value -----------------------------

impl FlexVal for Value {
    fn to_flex_val(&self) -> Value {
        self.clone()
    }
}

impl IntoFlexVal for Value {
    fn to_flex_val(self) -> Value {
        self
    }
}

// ----------------------------- Boolean | Option -----------------------------

impl FlexVal for bool {
    fn to_flex_val(&self) -> Value {
        Value::Boolean(self.clone())
    }
}

impl IntoFlexVal for bool {
    fn to_flex_val(self) -> Value {
        Value::Boolean(self)
    }
}

impl<T: FlexVal> FlexVal for Option<T> {
    fn to_flex_val(&self) -> Value {
        if let Some(v) = self {
            v.to_flex_val()
        } else {
            Value::Null
        }
    }
}

impl<T: IntoFlexVal> IntoFlexVal for Option<T> {
    fn to_flex_val(self) -> Value {
        if let Some(v) = self {
            v.to_flex_val()
        } else {
            Value::Null
        }
    }
}

// ----------------------------- Number -----------------------------

macro_rules! impl_trait_for_num {
    ($($t:ty)*) => ($(
        impl FlexVal for $t {
            fn to_flex_val(&self) -> Value {
                Value::Number(self.clone().into())
            }
        }

        impl IntoFlexVal for $t {
            fn to_flex_val(self) -> Value {
                Value::Number(self.into())
            }
        }
    )*)
}

impl_trait_for_num!(u8 u16 u32 i8 i16 i32 f32 f64);

// ----------------------------- String -----------------------------

impl FlexVal for &str {
    fn to_flex_val(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoFlexVal for &str {
    fn to_flex_val(self) -> Value {
        Value::String(self.to_string())
    }
}

impl FlexVal for String {
    fn to_flex_val(&self) -> Value {
        Value::String(self.clone())
    }
}

impl IntoFlexVal for String {
    fn to_flex_val(self) -> Value {
        Value::String(self)
    }
}

// ----------------------------- Array | Vector  -----------------------------

impl<T: FlexVal, const S: usize> FlexVal for [T; S] {
    fn to_flex_val(&self) -> Value {
        let mut arr = Array::new();
        for v in self {
            arr.push(v.to_flex_val())
        }
        Value::Array(arr)
    }
}

impl<T: IntoFlexVal, const S: usize> IntoFlexVal for [T; S] {
    fn to_flex_val(self) -> Value {
        let mut arr = Array::new();
        for v in self {
            arr.push(v.to_flex_val())
        }
        Value::Array(arr)
    }
}

impl<T: FlexVal> FlexVal for Vec<T> {
    fn to_flex_val(&self) -> Value {
        let mut arr = Array::new();
        for v in self {
            arr.push(v.to_flex_val())
        }
        Value::Array(arr)
    }
}

impl<T: FlexVal> IntoFlexVal for Vec<T> {
    fn to_flex_val(self) -> Value {
        let mut arr = Array::new();
        for v in self {
            arr.push(v.to_flex_val())
        }
        Value::Array(arr)
    }
}

impl<T: FlexVal> FlexVal for &[T] {
    fn to_flex_val(&self) -> Value {
        let mut arr = Array::new();
        for v in self.into_iter() {
            arr.push(v.to_flex_val())
        }
        Value::Array(arr)
    }
}

impl<T: IntoFlexVal> IntoFlexVal for &[T] {
    fn to_flex_val(self) -> Value {
        let mut arr = Array::new();
        for v in self.to_vec() {
            arr.push(v.to_flex_val());
        }
        Value::Array(arr)
    }
}

impl FlexVal for Array {
    fn to_flex_val(&self) -> Value {
        Value::Array(self.clone())
    }
}

impl IntoFlexVal for Array {
    fn to_flex_val(self) -> Value {
        Value::Array(self)
    }
}

// ----------------------------- Map  -----------------------------

impl FlexVal for Object {
    fn to_flex_val(&self) -> Value {
        Value::Object(self.clone())
    }
}

impl IntoFlexVal for Object {
    fn to_flex_val(self) -> Value {
        Value::Object(self)
    }
}
