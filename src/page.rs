use anyhow::Result;
use std::{fs, io::*};

pub struct Pages<const S: usize> {
    file: fs::File,
    /// Internal metadata size: 1 bytes
    metadata: [u8; S],
}

impl<const S: usize> Pages<S> {
    pub fn open(filepath: &str) -> Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filepath)?;

        if file.metadata()?.len() % S as u64 != 0 {
            panic!("Bad File");
        }

        let mut pages = Self {
            file,
            metadata: [0; S],
        };

        let mut metadata = pages.read(0);
        // is it newly created file ?
        if metadata.starts_with(&[0]) {
            metadata[0] = 1;
            pages.write(0, &metadata);
        }
        pages.metadata = metadata;
        Ok(pages)
    }

    pub fn metadata(&mut self) -> &mut [u8] {
        &mut self.metadata[1..]
    }

    pub fn sync_metadata(&mut self) {
        let buf = self.metadata;
        self.write(0, &buf);
    }

    pub fn read(&mut self, id: u64) -> [u8; S] {
        let mut buf = [0u8; S];
        self.file.seek(SeekFrom::Start(S as u64 * id)).unwrap();
        self.file.read(&mut buf).unwrap();
        buf
    }

    pub fn write(&mut self, id: u64, buf: &[u8; S]) {
        self.file.seek(SeekFrom::Start(S as u64 * id)).unwrap();
        self.file.write(buf).unwrap();
    }

    pub fn create(&mut self) -> u64 {
        let pos = self.file.seek(SeekFrom::End(0)).unwrap();
        self.file.set_len(pos + S as u64).unwrap();
        pos / S as u64
    }
}

#[test]
fn test_impl_Pages() {
    let mut pages: Pages<4096> = Pages::open("data.idx").unwrap();

    let a = pages.create();
    let b = pages.create();
    let c = pages.create();
    println!("{:#?}", a);
    println!("{:#?}", b);
    println!("{:#?}", c);
    fs::remove_file("data.idx");
}

