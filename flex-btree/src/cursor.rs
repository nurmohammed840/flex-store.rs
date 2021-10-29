use flex_page::Pages;
use std::io::Result;

use crate::{leaf::Leaf, node::Node};

pub struct Cursor<'a> {
    pages: &'a mut Pages<4096>,
    leaf: Leaf,
    index: usize,
}

impl Iterator for Cursor<'_> {
    type Item = Result<[u8; 8]>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.leaf.get_entrys().get(self.index) {
            self.index += 1;
            return Some(Ok(entry.value));
        }
        if self.leaf.right_child != 0 {
            return match self.pages.read(self.leaf.right_child as u64) {
                Ok(buf) => Node::from_bytes(buf).get_leaf().and_then(|leaf| {
                    self.index = 0;
                    self.leaf = leaf;
                    self.next()
                }),
                Err(err) => Some(Result::Err(err)),
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use crate::{BPlusTree, SetOption};

    use super::*;

    #[test]
    fn iter() -> Result<()> {
        let mut tree = BPlusTree::open("filepath")?;
        // max entry size is 255, So btree should split, into two leaf
        for i in 0..255 {
            tree.set(i, [i as u8; 8], SetOption::UpdateOrInsert)?;
        }
        let leaf = tree.find_leaf(tree.root_page_no, 0)?;
        let cursor = Cursor {
            index: 0,
            pages: &mut tree.pages,
            leaf,
        };
        for (i, res) in cursor.enumerate() {
            assert_eq!(res?, [i as u8; 8]);
        }

        std::fs::remove_file("filepath")?;
        Ok(())
    }
}
