use bin_layout::{Cursor, Decoder, Encoder};
use flex_page::Pages;
use std::{fs::File, io, path::Path};

pub trait PageNo:
    Into<u64> + From<u16> + std::fmt::Debug + Copy + Encoder + for<'de> Decoder<'de, io::Error>
{
}
impl PageNo for u16 {}
impl PageNo for u32 {}

#[derive(Debug, Encoder, Decoder, PartialEq)]
struct Info {
    is_dirty: bool,
    page_no_type: u8,
    block_size: u16,
}

pub struct PageManager<P: PageNo, const SIZE: usize> {
    pub pages: Pages<SIZE>,
    pub data: Vec<u8>,
    freelist_tail: P,
}

impl<P: PageNo, const SIZE: usize> PageManager<P, SIZE> {
    fn read_data(pages: &Pages<SIZE>, mut num: u64, data: &mut Vec<u8>) -> io::Result<()> {
        loop {
            let buf = pages.read(num)?;
            let mut c = Cursor::new(buf.as_ref());
            data.extend_from_slice(c.read_slice(SIZE - P::SIZE).unwrap());
            num = P::decoder(&mut c)?.into();
            if num == 0 {
                return Ok(());
            }
        }
    }

    fn write_data(pages: &Pages<SIZE>, mut num: u64, data: &[u8]) -> io::Result<()> {
        let size = SIZE - P::SIZE;
        let mut chunks = data.chunks(size).peekable();
        while let Some(chunk) = chunks.next() {
            let mut buf = pages.read(num)?;
            buf[..chunk.len()].copy_from_slice(chunk);

            let link = &mut buf[size..];
            let mut no = P::decode(link)?.into();
            if no == 0 && chunks.peek().is_some() {
                no = pages.alloc(1)?;
                link.copy_from_slice(P::from(no as u16).encode().as_ref());
            }
            pages.write(num, buf)?;
            num = no;
        }
        Ok(())
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let pages: Pages<SIZE> = Pages::open(file)?;
        let mut data: Vec<u8> = Vec::new();

        let mut buf = [0; SIZE];
        let info = Info {
            is_dirty: false,
            page_no_type: P::SIZE as u8,
            block_size: u16::try_from(SIZE - 1).expect("block size too large"),
        };
        let mut freelist_tail = P::from(1);

        if pages.len() == 0 {
            pages.alloc(2)?; // 1 for metadata, 1 for root node
            let mut raw = vec![];
            info.encoder(&mut raw);
            freelist_tail.encoder(&mut raw);
            buf[..raw.len()].copy_from_slice(&raw);
        } else {
            buf = pages.read(0)?;
            let mut c = Cursor::new(buf.as_ref());

            let meta: Result<_, ()> = Info::decoder(&mut c);
            assert_eq!(meta.unwrap(), info);
            freelist_tail = P::decoder(&mut c)?;

            let len = c.data.len() - (c.offset + P::SIZE);
            data.extend_from_slice(c.read_slice(len).unwrap());
            let next = P::decoder(&mut c)?.into();
            if next != 0 {
                Self::read_data(&pages, next, &mut data)?;
            }
        }
        buf[0] = 1; // is_dirty = true
        pages.write(0, buf)?;

        Ok(Self {
            pages,
            data,
            freelist_tail,
        })
    }
}

impl<P: PageNo, const SIZE: usize> Drop for PageManager<P, SIZE> {
    fn drop(&mut self) {
        let mut meta = vec![];
        Info {
            is_dirty: false,
            page_no_type: P::SIZE as u8,
            block_size: (SIZE - 1) as u16,
        }
        .encoder(&mut meta);
        self.freelist_tail.encoder(&mut meta);
        self.data.extend_from_slice(&meta);
        Self::write_data(&self.pages, 0, &meta).unwrap();
    }
}
