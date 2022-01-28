use std::{fs::remove_file, io::Result};

use page::Pages;
use tokio::test;

#[test]
async fn freelist() -> Result<()> {
    let pages: Pages<u16, 4096> = Pages::open("freelist")?;

    assert_eq!(pages.find_free_slot(), None);
    assert_eq!(pages.find_or_alloc_free_slot().await?, 1);

    assert_eq!(pages.free(1), true);
    assert_eq!(pages.free(1), false);

    assert_eq!(pages.find_free_slot(), Some(1));
    assert_eq!(pages.find_free_slot(), None);

    drop(pages);
    remove_file("freelist.db")?;
    remove_file("freelist.meta")
}

#[test]
async fn metadata() -> Result<()> {
    {
        let pages: Pages<u16, 4096> = Pages::open("metadata")?;

        assert_eq!(pages.set_metadata("hello", b"world"), None);
        assert_eq!(pages.set_metadata("hello", b"world!"), Some(b"world".to_vec()));

        let point = pages.alloc(3).await? as u16;
        pages.free(point + 0);
        pages.free(point + 1);
        pages.free(point + 2);
        assert_eq!(pages.freelist_len(), 3);
    }
    {
        let pages: Pages<u16, 4096> = Pages::open("metadata")?;
        assert_eq!(pages.get_metadata("hello"), Some(b"world!".to_vec()));
        assert_eq!(pages.freelist_len(), 3);
    }
    remove_file("metadata.db")?;
    remove_file("metadata.meta")
}
