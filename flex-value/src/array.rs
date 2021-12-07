use crate::Value;

pub trait Array {
    fn add<V: Into<Value>>(&mut self, value: V);
    fn set<V: Into<Value>>(&mut self, index: usize, value: V);
    fn fill_with<V: Into<Value>>(&mut self, value: V);
    fn resize_with<V: Into<Value>>(&mut self, new_len: usize, value: V);
    fn to_string(&self) -> String;
}

macro_rules! impl_method {
    [$name:ident => $m:ident $($arg:ident: $t:ty)*] => {
        #[inline]
        fn $m<V: Into<Value>>(&mut self, $($arg: $t,)* value: V) {
            self.$name($($arg,)* value.into());
        }
    };
}

impl Array for Vec<Value> {
    impl_method!(push => add);
    impl_method!(fill => fill_with);
    impl_method!(insert => set index: usize);
    impl_method!(resize => resize_with new_len: usize);

    fn to_string(&self) -> String {
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
