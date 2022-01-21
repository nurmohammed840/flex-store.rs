#![allow(warnings)]
#![feature(generic_const_exprs)]

use flex_page::Pages;
use std::fs::remove_file;
use std::io::Result;
use std::sync::Arc;

// #[test]
// fn basic() -> Result<()> {
//     {
//         let _pages = Pages::<u16, 4096>::open("tests/file.db")?;
//         // assert_eq!(pages.meta.data.len(), (4096 - 8));
//     }

//     remove_file("tests/file.db")
// }

// #[derive(Debug)]
// struct Bytes {
//     data: *mut [u8],
// }

// unsafe impl Send for Bytes {}

// impl Bytes {
//     fn new<const N: usize>(data: [u8; N]) -> Self {
//         Self { data: Box::into_raw(Box::new(data)) }
//     }
// }

// #[test]
// fn test_name() {
//     let bytes = Arc::new(Bytes::new([1; 5]));
//     // let data = bytes.data;
//     std::thread::spawn(move || {
//         unsafe {
//             let data = &mut *bytes.data;

//             // data[0] = 2;
//         }
//     });
//     // println!("{:?}", bytes.data);
// }
