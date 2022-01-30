use std::io::Result;
use std::fs::remove_file;

use flex_btree::{BTree, SetOption};
use SetOption::*;

async fn basic() -> Result<()> {
	let btree: BTree<u64, u32, u16, 4096> = BTree::open("basic")?;
    
    println!("{:#?}", btree.get(1).await?);
    println!("{:#?}", btree.set(1, 1, UpdateOrInsert).await?);
    println!("{:#?}", btree.get(1).await?);
    println!("{:#?}", btree.set(1, 2, UpdateOrInsert).await?);

    Ok(())
}

#[tokio::test]
async fn test() -> Result<()> {
    basic().await?;
    remove_file("basic.idx")?;
    remove_file("basic.idx.meta")
}