mod branch;
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

    pub fn get(&mut self, id: u64) -> Result<Option<[u8; 8]>> {
        fn next(page_no: u16, pages: &mut Pages<4096>, id: u64) -> Result<Option<[u8; 8]>> {
            match Node::from_bytes(pages.read(page_no as u64)?) {
                Node::Branch(b) => next(b.childs[b.lookup(id)], pages, id),
                Node::Leaf(leaf) => Ok(leaf.find_value(id)),
            }
        }
        next(self.root_page_no, &mut self.pages, id)
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
        let tuple = set(self.root_page_no, &mut self.pages, id, value, opt)?;
        if let Some((id, right_page_no)) = tuple.0 {
            let root = Branch::create_root(id, self.root_page_no, right_page_no);
            let page_no = self.pages.create()?;
            self.set_root(page_no as u16)?;
            self.pages.write(page_no, &Node::Branch(root).to_bytes())?;
        };
        Ok(tuple.1)
    }
}

fn set(
    page_no: u16,
    pages: &mut Pages<4096>,
    id: u64,
    value: [u8; 8],
    opt: SetOption,
) -> Result<(Option<(u64, u16)>, [u8; 8])> {
    let mut node = Node::from_bytes(pages.read(page_no as u64)?);
    let mut ret = None;
    let ret_value;
    match node {
        Node::Branch(ref mut branch) => {
            let i = branch.lookup(id);
            let tuple = set(branch.childs[i], pages, id, value, opt)?;
            ret_value = tuple.1;
            if let Some(value) = tuple.0 {
                branch.update(i, value);
                if branch.is_full() {
                    let (right, mid) = branch.split();
                    let page = pages.create()?;
                    pages.write(page, &Node::Branch(right).to_bytes())?;
                    ret = Some((mid, page as u16));
                }
                pages.write(page_no as u64, &node.to_bytes())?;
            }
        }
        Node::Leaf(ref mut left) => {
            ret_value = left.set_and_sort_entry(id, value, opt);
            if left.is_full() {
                let (mut right, mid) = left.split();
                let page = pages.create()?;

                right.left_child = page_no;
                left.right_child = page as u16;

                pages.write(page, &Node::Leaf(right).to_bytes())?;
                ret = Some((mid, page as u16));
            }
            pages.write(page_no as u64, &node.to_bytes())?;
        }
    };
    Ok((ret, ret_value))
}

#[cfg(test)]
mod tests {
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

        std::fs::remove_file("file.db")?;
        Ok(())
    }
}
