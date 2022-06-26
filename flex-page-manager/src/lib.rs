// #![allow(warnings)]
// use bin_layout::{Cursor, Decoder, Encoder};
// use flex_page::Pages;
// use std::{fmt, fs::File, io, path::Path};

// pub struct PageManager<P: PageNo, const SIZE: usize> {
//     pub pages: Pages<SIZE>,
//     pub data: Vec<u8>,
//     freelist_tail: P,
//     free_list: FreeList<P>,
// }

// impl<P: PageNo, const SIZE: usize> PageManager<P, SIZE> {
//     fn read_data(pages: &Pages<SIZE>, mut next: u64, data: &mut Vec<u8>) -> io::Result<()> {
//         loop {
//             let buf = pages.read(next)?;
//             let mut c = Cursor::new(buf.as_ref());
//             data.extend_from_slice(c.read_slice(SIZE - P::SIZE).unwrap());
//             next = P::decoder(&mut c)?.into();
//             if next == 0 {
//                 return Ok(());
//             }
//         }
//     }

//     fn write_data(pages: &mut Pages<SIZE>, mut next: u64, data: &[u8]) -> io::Result<()> {
//         let size = SIZE - P::SIZE;
//         let mut chunks = data.chunks(size).peekable();
//         while let Some(chunk) = chunks.next() {
//             let mut buf = pages.read(next)?;
//             buf[..chunk.len()].copy_from_slice(chunk);

//             let link = &mut buf[size..];
//             let mut num = P::decode(link)?.into();
//             if num == 0 && chunks.peek().is_some() {
//                 num = pages.alloc(1)?;
//                 link.copy_from_slice(P::from(num as u16).encode().as_ref());
//             }
//             pages.write(next, buf)?;
//             next = num;
//         }
//         Ok(())
//     }

//     pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
//         let file = File::options()
//             .read(true)
//             .write(true)
//             .create(true)
//             .open(path)?;

//         let mut pages: Pages<SIZE> = Pages::open(file)?;
//         let mut data: Vec<u8> = Vec::new();

//         let mut buf = [0; SIZE];
//         let info = Info {
//             is_dirty: false,
//             page_no_type: P::SIZE as u8,
//             block_size: u16::try_from(SIZE - 1).expect("block size too large"),
//         };
//         let mut freelist_tail = P::from(1);

//         if pages.len() == 0 {
//             pages.alloc(2)?; // 1 for metadata, 1 for freelist
//             let mut raw = vec![];
//             info.encoder(&mut raw);
//             freelist_tail.encoder(&mut raw);
//             buf[..raw.len()].copy_from_slice(&raw);
//         } else {
//             buf = pages.read(0)?;
//             let mut c = Cursor::new(buf.as_ref());

//             let meta: Result<_, ()> = Info::decoder(&mut c);
//             assert_eq!(meta.unwrap(), info);
//             freelist_tail = P::decoder(&mut c)?;

//             let len = c.data.len() - (c.offset + P::SIZE);
//             data.extend_from_slice(c.read_slice(len).unwrap());
//             let next = P::decoder(&mut c)?.into();
//             if next != 0 {
//                 Self::read_data(&pages, next, &mut data)?;
//             }
//         }
//         buf[0] = 1; // is_dirty = true
//         pages.write(0, buf)?;

//         let free_list = FreeList::decode(&pages.read(freelist_tail.into())?)?;
//         Ok(Self { pages, data, freelist_tail, free_list })
//     }

//     pub fn free(&self, no: P) {
//         // self.free_list.list.push(no);
//     }

//     // fn alloc(&self, count: u8) { }
// }

// impl<P: PageNo, const SIZE: usize> Drop for PageManager<P, SIZE> {
//     fn drop(&mut self) {
//         let mut meta = vec![];
//         Info {
//             is_dirty: false,
//             page_no_type: P::SIZE as u8,
//             block_size: (SIZE - 1) as u16,
//         }
//         .encoder(&mut meta);
//         self.freelist_tail.encoder(&mut meta);
//         self.data.extend_from_slice(&meta);
//         Self::write_data(&mut self.pages, 0, &meta).unwrap();
//     }
// }

// type List<Len, T> = bin_layout::Record<Len, Vec<T>>;

// pub trait PageNo:
//     Into<u64> + From<u16> + fmt::Debug + Copy + Encoder + for<'de> Decoder<'de, io::Error>
// {
// }
// impl PageNo for u16 {}
// impl PageNo for u32 {}

// #[derive(Debug, Encoder, Decoder, PartialEq)]
// struct Info {
//     is_dirty: bool,
//     page_no_type: u8,
//     block_size: u16,
// }

// #[derive(Encoder, Decoder)]
// struct FreeList<P> {
//     prev: P,
//     list: List<u16, P>,
// }

// const fn max_free_keys<P: PageNo, const SIZE: usize>() -> usize {
//     (SIZE - (P::SIZE + 2)) / P::SIZE
// }
