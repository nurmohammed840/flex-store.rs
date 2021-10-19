mod branch;
mod leaf;
mod node;
mod utill;

use branch::Branch;
use flex_page::Pages;
use node::Node;
use std::io::Result;

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

    pub fn insert(&mut self, id: u64, value: [u8; 8]) -> Result<()> {
        if let Some((id, right_page_no)) = insert(self.root_page_no, self, id, value)? {
            let root = Branch::create_root(id, self.root_page_no, right_page_no);
            let buf = Node::Branch(root).to_bytes();
            let new_page_no = self.pages.create()?;
            self.set_root_page(new_page_no as u16)?;
            self.pages.write(new_page_no, &buf)?;
        };
        Ok(())
    }
}

fn _a() {
   
}

fn insert(
    page_no: u16,
    this: &mut BPlusTree,
    id: u64,
    value: [u8; 8],
) -> Result<Option<(u64, u16)>> {
    let mut node = Node::from_bytes(this.pages.read(page_no as u64)?);

    match node {
        Node::Branch(ref mut branch) => {
            let mut handler = |i, left: &mut Branch| -> Result<Option<(u64, u16)>> {
                if let Some(value) = insert(left.childs[i], this, id, value)? {
                    left.update(value); // rodo
                    if left.is_full() {
                        let (right, mid) = left.split();
                        let page = this.pages.create()?;

                        this.pages.write(page, &Node::Branch(right).to_bytes())?;
                    }
                    // this.pages.write(page_no as u64, &node.to_bytes()).unwrap();
                }
                Ok(None)
            };

            let mut i = 0;
            for _id in branch.ids.into_iter().filter(|&id| id != 0) {
                if id == _id {
                    return Ok(None); // todo: duplicate id found! maybe should return error msg.
                }
                if id < _id {
                    return handler(i as usize, branch);
                }
                i += 1;
            }
            return handler(i as usize, branch);
            // Ok(None)
        }
        Node::Leaf(ref mut left) => {
            let mut ret = None;
            left.insert_and_sort_entry(id, value);
            if left.is_full() {
                let (mut right, mid) = left.split();
                let page = this.pages.create()?;

                right.left_child = page_no;
                left.right_child = page as u16;

                this.pages.write(page, &Node::Leaf(right).to_bytes())?;
                ret = Some((mid, page as u16));
            }

            this.pages.write(page_no as u64, &node.to_bytes())?;
            return Ok(ret);
        }
    }
}
