#![allow(warnings)]

mod meta;
mod page_no;

pub use page_no::*;
use std::{fs, io::*, marker::PhantomData};

pub struct Pages<T, const PAGE_NO_SIZE: usize, const S: usize> {
    file: fs::File,
    /// First byte of meta data indicate, If it was newly created file or not...
    metadata: [u8; S],
    _marker: PhantomData<T>,
}

impl<T, const PS: usize, const PAGE_SIZE: usize> Pages<T, PS, PAGE_SIZE>
where
    T: PageNo<PS>,
{
    pub fn open(path: &str) -> Result<Self> {
        let size_info = meta::size_info(PS, PAGE_SIZE);

        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                let mut file = fs::File::create(path)?;
                file.write(&size_info)?;
                file
            }
        };

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        if file.metadata()?.len() % PAGE_SIZE as u64 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut metadata = [0; PAGE_SIZE];
        file.read(&mut metadata)?;

        // if metadata.starts_with([needle]) {}

        Ok(Self {
            file,
            metadata,
            _marker: PhantomData,
        })
    }

    pub fn metadata(&mut self) -> &mut [u8] {
        &mut self.metadata[1..]
    }

    pub fn sync_metadata(&mut self) -> Result<()> {
        let buf = self.metadata;
        self.write(0, &buf)?;
        Ok(())
    }

    /// Clear Everything, Expect Matadata
    pub fn clear(&mut self) -> Result<()> {
        self.file.set_len(PAGE_SIZE as u64)?;
        Ok(())
    }

    // fn seek_page(file: &mut fs::File, page_no: T) -> Result<u64> {
    //     file.seek(SeekFrom::Start(PAGE_SIZE as u64 * page_no.into() as u64))
    // }

    // fn seek(&mut self, page_no: T) -> Result<u64> {
    //     Self::seek_page(&mut self.file, page_no)
    // }

    // fn read_page(file: &mut fs::File, page_no: T) -> Result<[u8; PAGE_SIZE]> {
    //     Self::seek_page(file, page_no)?;
    //     let mut buf = [0u8; PAGE_SIZE];
    //     file.read(&mut buf)?;
    //     Ok(buf)
    // }

    // pub fn _read(&mut self, page_no: T) -> Result<[u8; PAGE_SIZE]> {
    //     Self::read_page(&mut self.file, page_no)
    // }

    pub fn read(&mut self, page_no: u64) -> Result<[u8; PAGE_SIZE]> {
        let mut buf = [0u8; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(PAGE_SIZE as u64 * page_no))?;
        self.file.read(&mut buf)?;
        Ok(buf)
    }

    pub fn write(&mut self, page_no: u64, buf: &[u8; PAGE_SIZE]) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(PAGE_SIZE as u64 * page_no))?;
        self.file.write(buf)?;
        Ok(())
    }

    pub fn create(&mut self) -> Result<u64> {
        let len = self.file.seek(SeekFrom::End(0))?;
        self.file.set_len(len + PAGE_SIZE as u64)?; // todo: maybe remove this line?
        Ok(len / PAGE_SIZE as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_PATH: &str = "data.idx";

    #[test]
    fn all() -> Result<()> {
        // create_new_page
        {
            let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 1);
            assert_eq!(pages.create()?, 2);
        }
        {
            let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 3);
        }

        // read_write_metadata
        {
            let msg = b"Hello, World";
            {
                let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
                let raw_meta = pages.metadata();
                raw_meta[0..msg.len()].copy_from_slice(msg);
                pages.sync_metadata()?;
            }
            {
                let mut pages: Pages<u16, 2, 4096> = Pages::open(FILE_PATH)?;
                pages.clear()?; // It does't remove metadata
                let raw_meta = pages.metadata();
                assert_eq!(raw_meta[0..msg.len()], *msg);
            }
        }

        fs::remove_file(FILE_PATH)?;
        Ok(())
    }

    #[test]
    #[ignore = "todo: refector, Invalid page size"]
    fn read_write_data() -> Result<()> {
        let mut pages: Pages<u16, 2, 12> = Pages::open("data2.idx")?;
        let page_on = pages.create()?;

        let msg = b"Hello, World";
        pages.write(page_on, msg)?;
        assert_eq!(pages.read(page_on)?, *msg);

        fs::remove_file("data2.idx")?;
        Ok(())
    }
}
