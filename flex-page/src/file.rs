use std::{fs::File, io::Result, os};

#[cfg(target_family = "unix")]
pub use os::unix::fs::FileExt;

#[cfg(target_family = "wasm")]
pub use os::wasi::fs::FileExt;

#[cfg(target_family = "windows")]
pub trait FileExt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<usize>;
}
#[cfg(target_family = "windows")]
impl FileExt for File {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        os::windows::fs::FileExt::seek_read(self, buf, offset)
    }
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        os::windows::fs::FileExt::seek_write(self, buf, offset)
    }
}
