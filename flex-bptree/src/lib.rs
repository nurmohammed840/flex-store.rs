mod branch;
mod entry;
mod leaf;
mod node;

use bin_layout::{Cursor, Decoder, Encoder};
use flex_page::Pages;
use leaf::Leaf;

use std::{fs::File, io, path::Path};

use entry::Key;
use node::Node;

struct Link<K, V, const SIZE: usize> {
    id: u16,
    node: Node<K, V, SIZE>,
}

#[derive(Decoder, Encoder)]
pub struct Metadata {
    pub root_id: u16,
}

pub struct BPlusTree<K, V, const SIZE: usize> {
    pages: Pages<SIZE>,
    root: Link<K, V, SIZE>,
}

impl<K: Key, V: Key, const SIZE: usize> BPlusTree<K, V, SIZE> {
    /// #### _Blocking_
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let pages = Pages::open(file)?;

        let root = if pages.len() == 0 {
            pages.alloc(2)?; // 1 for metadata, 1 for root node
            Link {
                id: 1,
                node: Node::Leaf(Leaf::new()),
            }
        } else {
            let data = pages.read(0)?;
            let mut cursor = Cursor::new(data.as_ref());
            let metadata: Result<_, io::Error> = Metadata::decoder(&mut cursor);

            let id = metadata?.root_id;
            let buf = pages.read(id.into())?;
            Link {
                id,
                node: Node::decoder(buf.as_ref()),
            }
        };

        Ok(Self { pages, root })
    }

    fn set(&mut self, key: K, value: V) {}
}
