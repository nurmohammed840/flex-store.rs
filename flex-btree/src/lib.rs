mod branch;
mod cursor;
mod leaf;
mod node;
mod utill;

use flex_page::Pages;

use branch::Branch;
use node::Node;

use std::io::Result;

pub use leaf::SetOption;

pub struct BPlusTree {
    pages: Pages<4096>,
    root_page_no: u16,
}

impl BPlusTree {
    pub fn open(filepath: &str) -> Result<Self> {
        let mut pages = Pages::open(filepath)?;
        let raw = pages.metadata();
        let root_page_no = u16::from_ne_bytes([raw[0], raw[1]]);
        let mut tree = Self {
            pages,
            root_page_no,
        };
        if root_page_no == 0 {
            tree.set_root(1)?; // default to 1
        }
        Ok(tree)
    }

    fn set_root(&mut self, page_no: u16) -> Result<()> {
        self.root_page_no = page_no;
        self.pages.metadata()[0..2].copy_from_slice(&page_no.to_ne_bytes());
        self.pages.sync_metadata()?;
        Ok(())
    }

    fn find_leaf(&mut self, page_no: u16, id: u64) -> Result<leaf::Leaf> {
        match Node::from_bytes(self.pages.read(page_no as u64)?) {
            Node::Branch(b) => self.find_leaf(b.childs[b.lookup(id)], id),
            Node::Leaf(leaf) => Ok(leaf),
        }
    }

    pub fn get(&mut self, id: u64) -> Result<Option<[u8; 8]>> {
        Ok(self.find_leaf(self.root_page_no, id)?.find_value(id))
    }

    pub fn clear(&mut self) -> Result<()> {
        self.pages.clear()?;
        self.set_root(1)?;
        Ok(())
    }

    pub fn size(&self) {
        // todo
    }

    pub fn delete(&mut self) {
        // todo
    }

    pub fn set(&mut self, id: u64, value: [u8; 8], opt: SetOption) -> Result<[u8; 8]> {
        let tuple = self.set_next(self.root_page_no, id, value, opt)?;
        if let Some((id, right_page_no)) = tuple.0 {
            let root = Branch::create_root(id, self.root_page_no, right_page_no);
            let page_no = self.pages.create()?;
            self.set_root(page_no as u16)?;
            self.pages.write(page_no, &Node::Branch(root).to_bytes())?;
        };
        Ok(tuple.1)
    }

    fn set_next(
        &mut self,
        page_no: u16,
        id: u64,
        value: [u8; 8],
        opt: SetOption,
    ) -> Result<(Option<(u64, u16)>, [u8; 8])> {
        let mut node = Node::from_bytes(self.pages.read(page_no as u64)?);
        let mut ret = None;
        let ret_value;
        match node {
            Node::Branch(ref mut branch) => {
                let i = branch.lookup(id);
                let tuple = self.set_next(branch.childs[i], id, value, opt)?;
                ret_value = tuple.1;
                if let Some(value) = tuple.0 {
                    branch.update(i, value);
                    if branch.is_full() {
                        let (right, mid) = branch.split();
                        let page = self.pages.create()?;
                        self.pages.write(page, &Node::Branch(right).to_bytes())?;
                        ret = Some((mid, page as u16));
                    }
                    self.pages.write(page_no as u64, &node.to_bytes())?;
                }
            }
            Node::Leaf(ref mut left) => {
                ret_value = left.set_and_sort_entry(id, value, opt);
                if left.is_full() {
                    let (mut right, mid) = left.split();
                    let page = self.pages.create()?;

                    right.left_child = page_no;
                    left.right_child = page as u16;

                    self.pages.write(page, &Node::Leaf(right).to_bytes())?;
                    ret = Some((mid, page as u16));
                }
                self.pages.write(page_no as u64, &node.to_bytes())?;
            }
        };
        Ok((ret, ret_value))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    #[test]
    fn all() -> Result<()> {
        let mut btree = BPlusTree::open("file.db")?;

        btree.set(1, *b"Worked!!", SetOption::FindOrInsert)?;
        assert_eq!(
            *b"Worked!!", // Find
            btree.set(1, *b"Founded!", SetOption::FindOrInsert)?
        );

        btree.set(3, *b"NewValue", SetOption::UpdateOrInsert)?;
        assert_eq!(
            *b"Repleced", // Update
            btree.set(3, *b"Repleced", SetOption::UpdateOrInsert)?
        );

        assert_eq!(Some(*b"Repleced"), btree.get(3)?);

        std::fs::remove_file("file.db")?;
        Ok(())
    }

    #[test]
    #[ignore = "This test is only for debuging purpose"]
    fn debug_tree() -> Result<()> {
        let mut tree = BPlusTree::open("tree")?;
        tree.clear()?;
        for i in (1..=255).rev() {
            tree.set(i, [0; 8], SetOption::UpdateOrInsert)?;
        }
        std::fs::write("tree.txt", format_btree(&mut tree)?)?;
        std::fs::remove_file("tree")?;
        Ok(())
    }

    pub fn format_btree(tree: &mut BPlusTree) -> Result<String> {
        #[allow(dead_code)]
        #[derive(Debug)]
        enum TreeNode {
            Branch {
                ids: Vec<u64>,
                childs: Vec<TreeNode>,
            },
            Leaf(Vec<u64>),
        }
        fn build_tree(i: u16, pages: &mut Pages<4096>) -> Result<TreeNode> {
            Ok(match Node::from_bytes(pages.read(i as u64)?) {
                Node::Branch(b) => TreeNode::Branch {
                    ids: b.ids.into_iter().filter(|&r| r != 0).collect(),
                    childs: b
                        .childs
                        .into_iter()
                        .filter(|&i| i != 0)
                        .map(|i| build_tree(i, pages).unwrap())
                        .collect(),
                },
                Node::Leaf(l) => TreeNode::Leaf(
                    l.entrys[..(l.len as usize)]
                        .into_iter()
                        .map(|&f| f.id)
                        .collect(),
                ),
            })
        }
        fn prettier(txt: &mut std::str::Split<&str>, indent_lvl: usize) -> String {
            let mut arr = vec![];
            while let Some(chars) = txt.next() {
                match chars {
                    "{" | "[" => {
                        arr.push(chars.to_string());
                        arr.push("\n".to_string());
                        arr.push("\t".repeat(indent_lvl + 1));
                        arr.push(prettier(txt, indent_lvl + 1));
                    }
                    "}" | "]" => {
                        arr.push("\n".to_string());
                        arr.push("\t".repeat(indent_lvl - 1));
                        arr.push(chars.to_string());
                        return arr.concat();
                    }
                    _ => {
                        let tabs = &"\t".repeat(indent_lvl)[..];
                        let string = if chars.starts_with(", ") {
                            chars.replacen(", ", &["\n", tabs].concat(), 1)
                        } else {
                            chars.trim_start().replace("), ", &[")\n", tabs].concat())
                        };
                        arr.push(string);
                    }
                }
            }
            arr.concat()
        }
        let txt = prettier(
            &mut format!("{:?}", build_tree(tree.root_page_no, &mut tree.pages)?)
                .replace("([", "(")
                .replace("])", ")")
                .replace("{", "#{#")
                .replace("}", "#}#")
                .replace("[", "#[#")
                .replace("]", "#]#")
                .split("#"),
            0,
        );
        Ok(txt)
    }
}
