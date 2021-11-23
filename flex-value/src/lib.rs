mod array;
mod object;
mod prelude;

use std::{num::TryFromIntError, string::FromUtf8Error};

pub use array::Array;
use byte_seeker::ByteSeeker;
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

impl Value {
    pub fn new() -> Self {
        Self::Null
    }

    pub fn set<T: FlexVal>(&mut self, value: T) {
        *self = value.to_flex_val();
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Number(num) => Some(*num),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Array> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_arr_mut(&mut self) -> Option<&mut Array> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&Object> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_obj_mut(&mut self) -> Option<&mut Object> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

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
        let mut byte_seeker = ByteSeeker::new(&bytes);
        fn from(seeker: &mut ByteSeeker) -> Result<Value, FromUtf8Error> {
            let value = match seeker.next().unwrap() {
                0 => Value::Boolean(false),
                1 => Value::Boolean(true),
                2 => Value::Null,
                3 => Value::Number(f64::from_le_bytes(seeker.get_buf().unwrap())),
                4 => {
                    let len = u32::from_le_bytes(seeker.get_buf().unwrap());
                    let bytes = seeker.get_octets(len as usize).unwrap();
                    Value::String(String::from_utf8(bytes.to_vec())?)
                }
                5 => {
                    let len = u32::from_le_bytes(seeker.get_buf().unwrap());
                    let mut arr = Array::new();
                    for _ in 0..len {
                        arr.push(from(seeker)?)
                    }
                    Value::Array(arr)
                }
                6 => {
                    let len = u32::from_le_bytes(seeker.get_buf().unwrap());
                    let mut obj = Object::new();
                    for _ in 0..len {
                        let key_len = seeker.next().unwrap();
                        let bytes = seeker.get_octets(key_len as usize).unwrap();
                        obj.insert(&String::from_utf8(bytes.to_vec())?, from(seeker)?);
                    }
                    Value::Object(obj)
                }
                c => panic!("Invalid TypeCode: {}", c),
            };
            Ok(value)
        }
        from(&mut byte_seeker)
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
