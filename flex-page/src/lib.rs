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

        let mut metadata = pages.read(0)?;
        // is it newly created file ?
        if metadata.starts_with(&[0]) {
            metadata[0] = 1;
            pages.write(0, &metadata)?;
        }
        pages.metadata = metadata;
        Ok(pages)
    }

    pub fn metadata(&mut self) -> &mut [u8] {
        &mut self.metadata[1..]
    }

    pub fn sync_metadata(&mut self) -> Result<()> {
        let buf = self.metadata;
        self.write(0, &buf)?;
        Ok(())
    }

    pub fn read(&mut self, page_no: u64) -> Result<[u8; S]> {
        let mut buf = [0u8; S];
        self.file.seek(SeekFrom::Start(S as u64 * page_no))?;
        self.file.read(&mut buf)?;
        Ok(buf)
    }

    pub fn write(&mut self, page_no: u64, buf: &[u8; S]) -> Result<()> {
        self.file.seek(SeekFrom::Start(S as u64 * page_no))?;
        self.file.write(buf)?;
        Ok(())
    }

    pub fn create(&mut self) -> Result<u64> {
        let pos = self.file.seek(SeekFrom::End(0))?;
        self.file.set_len(pos + S as u64)?;
        Ok(pos / S as u64)
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
            let mut pages: Pages<4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 1);
            assert_eq!(pages.create()?, 2);
        }
        {
            let mut pages: Pages<4096> = Pages::open(FILE_PATH)?;
            assert_eq!(pages.create()?, 3);
        }

        // read_write_metadata
        {
            let msg = b"Hello, World";
            {
                let mut pages: Pages<4096> = Pages::open(FILE_PATH)?;
                let raw_meta = pages.metadata();
                raw_meta[0..msg.len()].copy_from_slice(msg);
                pages.sync_metadata()?;
            }
            {
                let mut pages: Pages<4096> = Pages::open(FILE_PATH)?;
                let raw_meta = pages.metadata();
                assert_eq!(raw_meta[0..msg.len()], *msg);
            }
        }

        fs::remove_file(FILE_PATH)?;
        Ok(())
    }

    #[test]
    fn read_write_data() -> Result<()> {
        let mut pages: Pages<12> = Pages::open("data2.idx")?;
        let page_on = pages.create()?;

        let msg = b"Hello, World";
        pages.write(page_on, msg)?;
        assert_eq!(pages.read(page_on)?, *msg);

        fs::remove_file("data2.idx")?;
        Ok(())
    }
}
