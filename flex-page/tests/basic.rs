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
