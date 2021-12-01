mod array;
mod object;
mod utils;
mod value;

pub use array::Array;
pub use object::Object;
pub use value::Value;

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
