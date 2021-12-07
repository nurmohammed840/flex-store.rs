use crate::{Array, Map, Object};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Map),
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
    #[inline]
    pub fn new() -> Self {
        Self::Null
    }
    impl_methods!(
        Arr: arr, as_arr -> Vec<Value>
        Str: str, as_str -> str
        Num: num, as_num -> f64
        Obj: obj, as_obj -> Map
        Bool: bool, as_bool -> bool
    );
    impl_methods_mut!(
        Str: str_mut, as_str_mut -> String
        Arr: arr_mut, as_arr_mut -> Vec<Value>
        Obj: obj_mut, as_obj_mut -> Map
    );
    #[inline]
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

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0.partial_cmp(r0),
            (Self::Num(l0), Self::Num(r0)) => l0.partial_cmp(r0),
            (Self::Str(l0), Self::Str(r0)) => l0.partial_cmp(r0),
            (Self::Arr(l0), Self::Arr(r0)) => l0.partial_cmp(r0),
            _ => None,
        }
    }
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
impl_costom_partial_eq!(Arr: Vec<Value>);
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
            Self::Str(s) => s[..].partial_cmp(other),
            _ => None,
        }
    }
}

macro_rules! convart_to {
    [$name:expr; $t:ty] => {
        impl From<$t> for Value {
            #[inline]
            fn from(v: $t) -> Self { $name(v) }
        }
    };
    [$name:expr => $($t:ty)*] => ($(
        impl From<$t> for Value {
            #[inline]
            fn from(v: $t) -> Self { $name(v.into()) }
        }
        impl From<&$t> for Value {
            #[inline]
            fn from(&v: &$t) -> Self { $name(v.into()) }
        }
    )*);

}
convart_to!(Value::Obj; Map);
convart_to!(Value::Str; String);
convart_to!(Value::Arr; Vec<Value>);
// Creates Value from borrowed data, usually by cloning...
convart_to!(Value::Str => &str);
convart_to!(Value::Bool => bool);
convart_to!(Value::Num => u8 u16 u32 i8 i16 i32 f32 f64);

impl<T: Into<Value>, const S: usize> From<[T; S]> for Value
// where
//     Vec<Value>: FromIterator<T>,
{
    #[inline]
    fn from(buf: [T; S]) -> Self {
        let mut arr = Vec::new();
        for ele in buf.into_iter() {
            arr.add(ele)
        }
        Self::Arr(arr)
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
