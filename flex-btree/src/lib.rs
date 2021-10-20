mod branch;
mod leaf;
mod node;
mod utill;

use branch::Branch;
use flex_page::Pages;
use node::Node;

use std::io::Result;
use utill::Results;

pub struct BPlusTree {
    pages: Pages<4096>,
    root_page_no: u16,
}

impl BPlusTree {
    pub fn open(filepath: &str) -> Result<Self> {
        let mut pages = Pages::open(filepath)?;
        let raw = pages.metadata();
        let root_id = u16::from_ne_bytes([raw[0], raw[1]]);
        let mut tree = Self {
            pages,
            root_page_no: root_id,
        };
        if root_id == 0 {
            tree.set_root_page(1)?; // default to 1
        }
        Ok(tree)
    }

    fn set_root_page(&mut self, no: u16) -> Result<()> {
        self.root_page_no = no;
        self.pages.metadata()[0..2].copy_from_slice(&no.to_ne_bytes());
        self.pages.sync_metadata()?;
        Ok(())
    }

    pub fn find(&self, _id: u64) {}

    pub fn insert(&mut self, id: u64, value: [u8; 8]) -> Results<()> {
        if let Some((id, right_page_no)) = insert(self.root_page_no, &mut self.pages, id, value)? {
            let root = Branch::create_root(id, self.root_page_no, right_page_no);
            let buf = Node::Branch(root).to_bytes();
            let new_page_no = self.pages.create()?;
            self.set_root_page(new_page_no as u16)?;
            self.pages.write(new_page_no, &buf)?;
        };
        Ok(())
    }
}

fn _find(page_no: u16, pages: &mut Pages<4096>, id: u64) -> Results<()> {
    match Node::from_bytes(pages.read(page_no as u64)?) {
        Node::Branch(branch) => {
            let _i = branch.lookup(id)?;
            // return handler(i as usize, branch);
        }
        Node::Leaf(_) => {}
    }
    Ok(())
}

type InsertResult = Results<Option<(u64, u16)>>;
fn insert(page_no: u16, pages: &mut Pages<4096>, id: u64, value: [u8; 8]) -> InsertResult {
    let mut node = Node::from_bytes(pages.read(page_no as u64)?);
    let mut ret = None;
    match node {
        Node::Branch(ref mut branch) => {
            let i = branch.lookup(id)?;
            if let Some(value) = insert(branch.childs[i], pages, id, value)? {
                branch.update(i, value); // rodo
                if branch.is_full() {
                    let (right, _mid) = branch.split();
                    let page = pages.create()?;

                    pages.write(page, &Node::Branch(right).to_bytes())?;
                }
                pages.write(page_no as u64, &node.to_bytes())?;
            }
        }
        Node::Leaf(ref mut left) => {
            left.insert_and_sort_entry(id, value);
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
    Ok(ret)
}
