#![feature(try_trait_v2)]

mod array;
mod object;
mod prelude;

use byte_seeker::{BytesReader, LittleEndian};
use std::{num::TryFromIntError, string::FromUtf8Error};

pub use array::Array;
pub use object::Object;
pub use prelude::*;

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
}

macro_rules! impl_methods {
    [$($method:ident : $x:ident,$y:ident -> $t:ty)*] => ($(
        #[inline]
        pub fn $x(&self) -> &$t { match self { Value::$method(v) => v, _ => panic!("Invalid Type") } }
        #[inline]
        pub fn $y(&self) -> Option<&$t> { match self { Value::$method(v) => Some(v), _ => None } }
    )*);
}
macro_rules! impl_methods_mut {
    [$($method:ident : $x:ident,$y:ident -> $t:ty)*] => ($(
        #[inline]
        pub fn $x(&mut self) -> &mut $t { match self { Value::$method(v) => v, _ => panic!("Invalid Type") } }
        #[inline]
        pub fn $y(&mut self) -> Option<&mut $t> { match self { Value::$method(v) => Some(v), _ => None } }
    )*);
}

impl Value {
    pub fn new() -> Self {
        Self::Null
    }

    pub fn set<T: FlexVal>(&mut self, value: T) {
        *self = value.to_flex_val();
    }

    impl_methods!(
        Array: arr, as_arr -> Array
        String: str, as_str -> str
        Number: num, as_num -> f64
        Object: obj, as_obj -> Object
        Boolean: bool, as_bool -> bool
    );

    impl_methods_mut!(
        Array: arr_mut, as_arr_mut -> Array
        Object: obj_mut, as_obj_mut -> Object
    );

    pub fn to_byte(&self) -> Result<Vec<u8>, TryFromIntError> {
        let bytes = match self {
            Value::Boolean(b) => match b {
                false => vec![0],
                true => vec![1],
            },
            Value::Null => vec![2],
            Value::Number(num) => [vec![3], num.to_le_bytes().to_vec()].concat(),
            Value::String(string) => {
                let len: u32 = string.len().try_into()?;
                [
                    vec![4],
                    len.to_le_bytes().to_vec(),
                    string.clone().into_bytes(),
                ]
                .concat()
            }
            Value::Array(arr) => {
                let len: u32 = arr.len().try_into()?;
                let mut bytes = [vec![5], len.to_le_bytes().to_vec()].concat();
                for value in arr.iter() {
                    bytes.append(&mut value.to_byte()?);
                }
                bytes
            }
            Value::Object(ref obj) => {
                let len: u32 = obj.len().try_into()?;
                let mut bytes = [vec![6], len.to_le_bytes().to_vec()].concat();
                for (key, value) in obj.iter() {
                    let key_len: u8 = key.len().try_into()?;
                    bytes.push(key_len);
                    bytes.append(&mut key.clone().into_bytes());
                    bytes.append(&mut value.to_byte()?);
                }
                bytes
            }
        };
        Ok(bytes)
    }

    pub fn from_byte(bytes: Vec<u8>) -> Result<Value, FromUtf8Error> {
        fn from(reader: &mut BytesReader) -> Result<Value, FromUtf8Error> {
            let value = match reader.next().unwrap() {
                0 => Value::Boolean(false),
                1 => Value::Boolean(true),
                2 => Value::Null,
                3 => Value::Number(reader.read()),
                4 => {
                    let len: u32 = reader.read();
                    let bytes = reader.bytes(len as usize);
                    Value::String(String::from_utf8(bytes.to_vec())?)
                }
                5 => {
                    let len: u32 = reader.read();
                    let mut arr = Array::new();
                    for _ in 0..len {
                        arr.push(from(reader)?)
                    }
                    Value::Array(arr)
                }
                6 => {
                    let len: u32 = reader.read();
                    let mut obj = Object::new();
                    for _ in 0..len {
                        let key_len = reader.next().unwrap();
                        let bytes = reader.bytes(key_len as usize);
                        obj.insert(&String::from_utf8(bytes.to_vec())?, from(reader)?);
                    }
                    Value::Object(obj)
                }
                c => panic!("Invalid TypeCode: {}", c),
            };
            Ok(value)
        }
        from(&mut BytesReader::new(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_byte_from_byte() {
        let favorite = obj! {
            "Princess" => ["raya", "moana", "dora"],
            "AnimatedMovies" => [
                obj! { "name" => "Zootopia", "year" => 2016 },
                obj! { "name" => "The Croods", "year" => 2013 },
                obj! { "name" => "Big Hero 6", "year" => 2014 },
            ],
            "supportedTypes" => arr![Value::Null, 1, true, "string", ["array"], obj!{ "type" => "object" }]
        };
        let json = favorite.to_string();
        let bytes = favorite.to_flex_val().to_byte().unwrap();
        let value = Value::from_byte(bytes).unwrap();
        assert_eq!(value.to_string(), json);
        println!("{}", value.to_string());
    }
}
