use crate::{Array, Object};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Array),
    Obj(Object),
}
macro_rules! impl_costom_partial_eq {
    [$name:ident: $t:ty] => (
        impl PartialEq<$t> for Value {
            #[inline]
            fn eq(&self, r0: &$t) -> bool { match self {Self::$name(l0) => l0 == r0, _ => false } }
        }
        impl PartialOrd<$t> for Value {
            #[inline]
            fn partial_cmp(&self, other: &$t) -> Option<std::cmp::Ordering> { match self { Self::$name(l0) => l0.partial_cmp(other), _ => None } }
        }
    );
    [$($t:ty)*] => ($(
        impl PartialEq<$t> for Value {
            #[inline]
            fn eq(&self, r0: &$t) -> bool { self == &Self::from(r0) }
        }
        impl PartialOrd<$t> for Value {
            #[inline]
            fn partial_cmp(&self, other: &$t) -> Option<std::cmp::Ordering> { self.partial_cmp(&Self::from(other)) }
        }
    )*);
}
impl_costom_partial_eq!(Bool: bool);
impl_costom_partial_eq!(Num: f64);
impl_costom_partial_eq!(Str: String);
impl_costom_partial_eq!(Arr: Array);
impl_costom_partial_eq!(Obj: Object);
impl_costom_partial_eq!(u8 u16 u32 i8 i16 i32 f32);

impl PartialEq<&str> for Value {
    fn eq(&self, r0: &&str) -> bool {
        match self {
            Self::Str(l0) => l0 == r0,
            _ => false,
        }
    }
}
impl PartialOrd<&str> for Value {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        match self {
            Self::Str(l0) => l0.partial_cmp(&other.to_string()),
            _ => None,
        }
    }
}

crate::utils::derives!(Value: Display);
crate::utils::derives!(Value: New);

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

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}
impl Value {
    impl_methods!(
        Arr: arr, as_arr -> Array
        Str: str, as_str -> str
        Num: num, as_num -> f64
        Obj: obj, as_obj -> Object
        Bool: bool, as_bool -> bool
    );
    impl_methods_mut!(
        Str: str_mut, as_str_mut -> String
        Arr: arr_mut, as_arr_mut -> Array
        Obj: obj_mut, as_obj_mut -> Object
    );
    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Num(num) => num.to_string(),
            Value::Str(string) => format!("{:?}", string),
            Value::Arr(arr) => arr.to_string(),
            Value::Obj(obj) => obj.to_string(),
        }
    }
}

macro_rules! convart_to {
    [$name:expr; $t:ty] => {
        impl From<$t> for Value {
            #[inline]
            fn from(val: $t) -> Self { $name(val.into()) }
        }
    };
    [$name:expr => $($t:ty)*] => ($(
        convart_to!($name; $t);
        impl From<&$t> for Value {
            #[inline]
            fn from(val: &$t) -> Self { $name(val.clone().into()) }
        }
    )*);
}
convart_to!(Value::Arr; Array);
convart_to!(Value::Obj; Object);
// Creates Value from borrowed data, usually by cloning...
convart_to!(Value::Bool => bool);
convart_to!(Value::Str => &str String);
convart_to!(Value::Num => u8 u16 u32 i8 i16 i32 f32 f64);

impl<T: Into<Value>, const S: usize> From<[T; S]> for Value {
    #[inline]
    fn from(val: [T; S]) -> Self {
        Self::Arr(val.into())
    }
}
impl<T: Into<Value>> From<Vec<T>> for Value {
    #[inline]
    fn from(val: Vec<T>) -> Self {
        Self::Arr(val.into())
    }
}
impl<T: Into<Value>> From<Option<T>> for Value {
    #[inline]
    fn from(val: Option<T>) -> Self {
        if let Some(v) = val {
            v.into()
        } else {
            Self::Null
        }
    }
}