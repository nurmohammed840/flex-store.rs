macro_rules! extends {
    [$t:ty : $target:ty] => {
        impl std::ops::Deref for $t {
            type Target = $target;
            #[inline]
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl std::ops::DerefMut for $t {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    };
}
macro_rules! derives {
    // This macro called `to_string` method from implementor struct...
    // Make sure that, struct has a `to_string` methods, Or It might cause stack-overflow.
    // Because, Display trait auto implement `to_string` methods, if there is no `to_string` found in implementor struct.
    [$t:ty: Display] => {
        impl std::fmt::Display for $t {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.to_string()) }
        }
    };
    [$t:ty: New] => {
        impl $t {
            #[inline]
            pub fn new() -> Self { Self::default() }
        }
    };
}

pub(crate) use extends;
pub(crate) use derives;


/* 
use byte_seeker::{BytesReader, LittleEndian};
use std::{num::TryFromIntError, string::FromUtf8Error};

impl Value {
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
*/