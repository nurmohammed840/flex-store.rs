use crate::PageNo;

#[derive(PartialEq, Debug)]
pub struct SizeInfo {
    page_no_size: u8,
    page_size: u32,
}

impl SizeInfo {
    pub fn new(page_no_size: usize, page_size: usize) -> Self {
        assert!(page_size >= 64, "Page size should >= 64 bytes");
        assert!(page_size < 16777216, "Page size should < 16mb");
        Self {
            page_no_size: page_no_size as u8,
            page_size: page_size as u32,
        }
    }

    pub fn to_buf(&self) -> [u8; 4] {
        let [x, y, z, _] = self.page_size.to_le_bytes();
        [self.page_no_size, x, y, z]
    }

    pub fn from(buf: [u8; 4]) -> Self {
        let [a, b, c, d] = buf;
        Self {
            page_no_size: a,
            page_size: u32::from_le_bytes([b, c, d, 0]),
        }
    }
}

pub struct Meta<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> {
    page_no_size: u8,
    page_size: u32,
    next: P,
}

impl<P: PageNo<PS>, const PS: usize, const PAGE_SIZE: usize> Meta<P, PS, PAGE_SIZE> {
    pub fn new() -> Self {
        Self {
            next: todo!(),
            page_no_size: todo!(),
            page_size: todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn size_info() {
        let size_info = SizeInfo::new(3, 4096);
        assert!(size_info == SizeInfo::from(size_info.to_buf()));
    }
}
