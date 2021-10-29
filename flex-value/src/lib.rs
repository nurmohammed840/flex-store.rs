mod array;
mod map;
mod prelude;
mod utils;

use array::Array;
use map::Map;
use utils::ByteSeeker;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Null,
    Number(f64),
    String(String),
    Array(Array),
    Map(Map),
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(boolean) => Some(boolean),
            _ => None,
        }
    }
    pub fn as_num(&self) -> Option<f64> {
        match *self {
            Value::Number(num) => Some(num),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref string) => Some(&string),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Array> {
        match *self {
            Value::Array(ref arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_arr_mut(&mut self) -> Option<&mut Array> {
        match *self {
            Value::Array(ref mut arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&Map> {
        match *self {
            Value::Map(ref obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_obj_mut(&mut self) -> Option<&mut Map> {
        match *self {
            Value::Map(ref mut obj) => Some(obj),
            _ => None,
        }
    }

    pub fn to_byte(&self) -> Vec<u8> {
        match *self {
            Value::Bool(b) => match b {
                false => vec![0],
                true => vec![1],
            },
            Value::Null => vec![2],
            Value::Number(num) => [vec![3], num.to_le_bytes().to_vec()].concat(),
            Value::String(_) => todo!(),
            Value::Array(_) => todo!(),
            Value::Map(ref obj) => {
                let len: u16 = obj
                    .len()
                    .try_into()
                    .expect(&format!("To Much Items! Max Capacity: {}", u16::MAX));

                let mut bytes = [vec![6], len.to_le_bytes().to_vec()].concat();
                for (key, value) in obj.iter() {
                    let key_len: u8 = key
                        .len()
                        .try_into()
                        .expect(&format!("Key({}) length should less then 255", key));

                    bytes.push(key_len);
                    bytes.append(&mut key.clone().into_bytes());
                    bytes.append(&mut value.to_byte());
                }
                bytes
            }
        }
    }

    pub fn from_byte(seeker: &mut ByteSeeker) -> Self {
        match seeker.first() {
            0 => Value::Bool(false),
            1 => Value::Bool(true),
            2 => Value::Null,
            3 => Value::Number(f64::from_le_bytes(seeker.get_buf())),
            4 => {
                let _len = u32::from_le_bytes(seeker.get_buf());
                todo!()
            }
            5 => {
                let _len = u16::from_le_bytes(seeker.get_buf());
                todo!()
            }
            6 => {
                let len = u16::from_le_bytes(seeker.get_buf());
                let mut map = Map::new();
                for _ in 0..len {
                    let key_len = seeker.first();
                    let vec = seeker.get_vec(key_len as usize);

                    map.insert(&String::from_utf8(vec).unwrap(), Value::from_byte(seeker));
                }
                Value::Map(map)
            }
            c => panic!("Invalid TypeCode: {}", c),
        }
    }
}

#[cfg(test)]
mod tests {
    // use json::JsonValue;
    // use serde_json::{json, Value};


    #[test]
    fn it_works() {
        playground();
    }

    fn playground() -> Option<()> {
        Some(())
    }
}
