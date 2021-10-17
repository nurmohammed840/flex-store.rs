use crate::{branch::Branch, node::Node, page::Pages, utill::insert_within_slice};

pub struct BPlusTree {
    pages: Pages<4096>,
    root_id: u16,
}

impl BPlusTree {
    pub fn open(filepath: &str) -> Self {
        let mut pages = Pages::open(filepath).unwrap();
        let raw = pages.metadata();
        let root_id = u16::from_le_bytes([raw[0], raw[1]]);
        let mut tree = Self { pages, root_id };
        if root_id == 0 {
            tree.set_root_id(1); // default to 1
        }
        tree
    }

    pub fn set_root_id(&mut self, id: u16) {
        self.root_id = id;
        self.pages.metadata()[0..2].copy_from_slice(&id.to_le_bytes());
        self.pages.sync_metadata();
    }

    pub fn insert(&mut self, id: u64, value: [u8; 8]) {
        if let Some((id, right_page_no)) = insert(self.root_id, self, id, value) {
            let mut branch = Branch::new();
            branch.ids[0] = id;
            branch.childs[0] = self.root_id;
            branch.childs[1] = right_page_no;
            let buf = Node::Branch(branch).to_bytes();

            let new_page_no = self.pages.create();
            self.set_root_id(new_page_no as u16);
            self.pages.write(new_page_no, &buf);
        };
    }
}

fn insert(page_no: u16, this: &mut BPlusTree, id: u64, value: [u8; 8]) -> Option<(u64, u16)> {
    let mut node = Node::from_bytes(this.pages.read(page_no as u64));

    match node {
        Node::Branch(ref mut banch) => {
            let mut handler = |page_no, banch: &mut Branch| {
                if let Some((mid, p)) = insert(banch.childs[page_no], this, id, value) {
                    // insert_within_slice(&mut banch.ids, mid, page_no);
                    // insert_within_slice(&mut banch.childs, p, page_no + 1);
                }
            };

            let mut i = 0;
            for &ele in banch.ids.clone().iter().filter(|&&a| a != 0) {
                if id < ele {
                    handler(i, banch);
                }
                i += 1;
            }
            handler(i, banch);
        }

        Node::Leaf(ref mut left) => {
            left.insert_and_sort_entry(id, value);
            if left.is_full() {
                let (mut right, mid_elem) = left.split();
                let new_page_no = this.pages.create();

                right.left_child = page_no as u16;
                left.right_child = new_page_no as u16;

                this.pages.write(new_page_no, &Node::Leaf(right).to_bytes());
                this.pages.write(page_no as u64, &node.to_bytes());
                return Some((mid_elem, new_page_no as u16));
            }
            this.pages.write(page_no as u64, &node.to_bytes());
        }
    }
    return None;
}

#[test]
#[ignore = "no_reason"]
fn print_bp_tree() {
    use std::fs;

    #[derive(Debug)]
    enum Log {
        Branch { child: Vec<Log> },
        Leaf { ids: u8 },
    }

    let mut tree = BPlusTree::open("data.hex");

    fn next(page_no: u16, this: &mut BPlusTree) -> Log {
        match Node::from_bytes(this.pages.read(page_no as u64)) {
            Node::Branch(b) => {
                let mut child = vec![];
                for i in b.childs {
                    let node = next(i, this);
                    child.push(node);
                }
                return Log::Branch { child };
            }
            Node::Leaf(node) => return Log::Leaf { ids: node.len },
        }
    };

    println!("{:#?}", next(tree.root_id, &mut tree));
    fs::remove_file("data.hex").unwrap();
}
