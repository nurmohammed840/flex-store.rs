use crate::page::Pages;
use std::mem;

#[derive(Clone, Copy, Default)]
struct Entry {
    id: u64,
    value: [u8; 8],
}

struct Leaf {
    len: u8,
    entrys: [Entry; 255],
    left_child: u16,
    right_child: u16,
}

impl Leaf {
    fn new() -> Self {
        Self {
            len: Default::default(),
            entrys: [Default::default(); 255],
            left_child: Default::default(),
            right_child: Default::default(),
        }
    }

    fn insert_and_sort_entry(&mut self, id: u64, value: [u8; 8]) {
        let mut pos = self.len as usize;
        while pos > 0 && self.entrys[pos - 1].id > id {
            self.entrys[pos] = self.entrys[pos - 1];
            pos -= 1;
        }
        self.entrys[pos].id = id;
        self.entrys[pos].value = value;
        self.len += 1;
    }

    fn is_full(&self) -> bool {
        self.len == 255
    }

    fn split(&mut self) -> Leaf {
        let mut right = Leaf::new();
        self.entrys[127..]
            .iter_mut()
            .zip(&mut right.entrys[127..])
            .for_each(|(l, r)| {
                let t = *r;
                *r = *l;
                *l = t;
            });

        right
    }
}

enum Node {
    Leaf(Leaf),
    Branch { ids: [u64; 409], childs: [u16; 410] },
}

impl Node {
    fn to_bytes(self) -> [u8; 4096] {
        unsafe { mem::transmute::<Self, [u8; 4096]>(self) }
    }
    fn from_bytes(bytes: [u8; 4096]) -> Self {
        unsafe { mem::transmute::<[u8; 4096], Self>(bytes) }
    }
}

struct BPlusTree {
    pages: Pages<4096>,
    root_id: u16,
}

impl BPlusTree {
    fn open(filepath: &str) -> Self {
        let mut pages = Pages::open(filepath).unwrap();
        let raw = pages.metadata();
        let root_id = u16::from_le_bytes([raw[0], raw[1]]);
        let mut tree = Self { pages, root_id };
        if root_id == 0 {
            tree.set_root_id(1); // default to 1
        }
        tree
    }

    fn set_root_id(&mut self, id: u16) {
        self.root_id = id;
        self.pages.metadata()[0..2].copy_from_slice(&id.to_le_bytes());
        self.pages.sync_metadata();
    }

    fn insert_(&mut self, id: u64, value: [u8; 8]) {
        fn next(page_no: u16, this: &mut BPlusTree, id: u64, value: [u8; 8]) -> Option<(u64, u16)> {
            let mut node = Node::from_bytes(this.pages.read(page_no as u64));

            match node {
                Node::Branch {
                    ref mut ids,
                    ref mut childs,
                } => {
                    let ids_clone = ids.clone();

                    let mut handler = |page_no| {
                        if let Some((mid, p)) = next(childs[page_no], this, id, value) {
                            insert_within_slice(ids, mid, page_no);
                            insert_within_slice(childs, p, page_no + 1);

                            if *ids.last().unwrap() != 0 {
                                let mut right_branch_ids = [0; 409];
                                let mut right_branch_childs = [0; 410];

                                

                                let mut right_branch = Node::Branch {
                                    ids: right_branch_ids,
                                    childs: right_branch_childs,
                                };
                                let buf = right_branch.to_bytes();
                            }
                        }
                        return None;
                    };

                    let mut i = 0;
                    for &ele in ids_clone.into_iter().filter(|&&v| v != 0) {
                        if id < ele {
                            return handler(i);
                        }
                        i += 1;
                    }
                    return handler(i); // last
                }

                Node::Leaf(ref mut left) => {
                    left.insert_and_sort_entry(id, value);
                    if left.is_full() {
                        let mut right = left.split();
                        let new_page_no = this.pages.create();

                        right.left_child = page_no as u16;
                        left.right_child = new_page_no as u16;

                        let mid_elem = right.entrys[0].id;

                        this.pages.write(new_page_no, &Node::Leaf(right).to_bytes());
                        this.pages.write(page_no as u64, &node.to_bytes());
                        return Some((mid_elem, new_page_no as u16));
                    }
                    this.pages.write(page_no as u64, &node.to_bytes());
                }
            }
            return None;
        }
        if let Some((id, right_page_no)) = next(self.root_id, self, id, value) {
            let mut ids = [0; 409];
            let mut childs = [0; 410];

            ids[0] = id;
            childs[0] = self.root_id;
            childs[1] = right_page_no;

            let mut branch = Node::Branch { ids, childs };
            let buf = branch.to_bytes();

            let new_page_no = self.pages.create();
            self.set_root_id(new_page_no as u16);
            self.pages.write(new_page_no, &buf);
        };
    }
}
/// Insert a value on index. move all elemment to right (advance 1).
fn insert_within_slice<T: Copy>(arr: &mut [T], value: T, offset: usize) {
    let last_idx = arr.len() - 1;
    arr.copy_within(offset..last_idx, offset + 1);
    arr[offset] = value;
}

#[test]
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
            Node::Branch { ids, childs } => {
                let mut child = vec![];
                for i in childs {
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

#[test]
fn test_set_root_id() {
    std::fs::remove_file("data.hex").unwrap();
    {
        let mut tree = BPlusTree::open("data.hex");
        assert_eq!(tree.root_id, 1);
        tree.set_root_id(10);
    }
    {
        let tree = BPlusTree::open("data.hex");
        assert_eq!(tree.root_id, 10);
    }
}
