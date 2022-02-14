use std::fs::remove_file;
use std::io::Result;

use flex_btree::{BTree, SetOption};
use SetOption::*;

async fn basic() -> Result<()> {
	{
		let btree: BTree<u64, u64, u16, 64> = BTree::open("basic")?;
		for i in 1..=5000 {
			btree.set(i, 1, UpdateOrInsert).await.unwrap();
		}
		println!("{:#?}", btree.first_key_value().await?);
		println!("{:#?}", btree.last_key_value().await?);
	}
	Ok(())
}

#[tokio::test]
async fn test() -> Result<()> {
	basic().await?;
	remove_file("basic")?;
	Ok(())
}
