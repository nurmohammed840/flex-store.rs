
#[derive(Debug, Clone)]
struct TrieMap<T> {
    key: char,
    value: Option<T>,
    childs: Vec<(TrieMap<T>)>,
}

impl<T> TrieMap<T> {
    fn new() -> Self {
        Self {
            key: char::default(),
            value: None,
            childs: vec![],
        }
    }

    fn find(&mut self, key: &str) -> &Option<T> {
        let mut t = self;
        for c in key.chars() {
            match t.childs.binary_search_by_key(&c, |t| t.key) {
                Ok(i) => {
                    t = &mut t.childs[i];
                }
                Err(_) => return &None,
            };
            // t = match t.childs.binary_search_by_key(&c, |t| t.key) {
            //     Ok(i) => &mut t.childs[i],
            //     Err(i) => {

            //     }
            // }
        }
        &t.value
    }

    fn insert(&mut self, key: &str, value: T) {
        let mut t = self;
        for c in key.chars() {
            t = match t.childs.binary_search_by_key(&c, |t| t.key) {
                Ok(i) => &mut t.childs[i],
                Err(i) => {
                    let mut e = Self::new();
                    e.key = c;
                    t.childs.insert(i, e);
                    &mut t.childs[i]
                }
            }
        }
        t.value = Some(value);
    }
}

#[test]
fn test_name() {
    let mut f = TrieMap::new();
    f.insert("boy", "value");
    f.insert("cat", "value");
    f.insert("apple", "value");
    println!("{:#?}", f);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
