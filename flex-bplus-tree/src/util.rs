use std::mem;

pub trait ToFromBytes {
    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Result<&Self, &str>
    where
        Self: Sized,
    {
        if bytes.len() < mem::size_of::<Self>() {
            return Err("InsufficientBytes");
        }
        if bytes.as_ptr().align_offset(mem::align_of::<Self>()) != 0 {
            return Err("InsufficientAlignment");
        }
        Ok(mem::transmute::<*const u8, &Self>(bytes.as_ptr()))
    }
    #[inline]
    unsafe fn to_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        let pointer = self as *const Self as *const u8;
        std::slice::from_raw_parts(pointer, mem::size_of::<Self>())
    }
}

pub trait Key {}

impl Key for u16 {}
impl Key for u32 {}
impl<const S: usize> Key for [u8; S] {}
