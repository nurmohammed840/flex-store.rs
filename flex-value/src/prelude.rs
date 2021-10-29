use crate::{Array, Map, Value};

/*
    It contain
*/

pub trait FlexVal: IntoFlexVal {
    fn to_flex_val(&self) -> Value;
}

pub trait IntoFlexVal: Clone {
    fn to_flex_val(self) -> Value;
}

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
        Value::Bool(self.clone())
    }
}

impl IntoFlexVal for bool {
    fn to_flex_val(self) -> Value {
        Value::Bool(self)
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

impl FlexVal for Map {
    fn to_flex_val(&self) -> Value {
        Value::Map(self.clone())
    }
}

impl IntoFlexVal for Map {
    fn to_flex_val(self) -> Value {
        Value::Map(self)
    }
}

