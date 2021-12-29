use std::{fs::File, io::Result, os};

#[cfg(target_family = "windows")]
mod sys {
    use super::*;
    use os::windows::fs::FileExt;
    
    pub fn read(this: &File, buf: &mut [u8], offset: u64) -> Result<usize> {
        FileExt::seek_read(this, buf, offset)
    }
    pub fn write(this: &File, buf: &[u8], offset: u64) -> Result<usize> {
        FileExt::seek_write(this, buf, offset)
    }
}
#[cfg(target_family = "unix")]
mod sys {
    use super::*;
    use os::unix::fs::FileExt;

    pub fn read(this: &File, buf: &mut [u8], offset: u64) -> Result<usize> {
        FileExt::read_at(this, buf, offset)
    }
    pub fn write(this: &File, buf: &[u8], offset: u64) -> Result<usize> {
        FileExt::write_at(this, buf, offset)
    }
}

pub trait FileExt {
    fn read(&self, buf: &mut [u8], offset:  u64) -> Result<usize>;
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize>;
}

impl FileExt for File {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        sys::read(self, buf, offset)
    }
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize> {
        sys::write(self, buf, offset)
    }
}