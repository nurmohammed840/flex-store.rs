use std::{fs::remove_file, io::Result};

// use flex_btree::SetOption;
// use SetOption::*;

type BTree = flex_btree::BPlusTree<u64, u16, 64>;

#[test]
fn open_file() -> Result<()> {
	let _ = remove_file("open_file");
	{
		let _btree = BTree::open("open_file")?;
		assert_eq!(
			"The file is already opened.",
			BTree::open("open_file").err().unwrap().to_string()
		);
	}
	assert_eq!(
		"Expected: MetaInfo { key_size: 8, value_size: 2, block_size: 64 }, but got: MetaInfo { key_size: 4, value_size: 4, block_size: 128 }",
		flex_btree::BPlusTree::<u32, u32, 128>::open("open_file")
			.err()
			.unwrap()
			.to_string()
	);
	assert!(BTree::open("open_file").err().is_none());
	remove_file("open_file")
}
