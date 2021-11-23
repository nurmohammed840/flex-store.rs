use byte_seeker::ByteSeeker;

use crate::PageNo;
// pub struct MataDatas<const S: usize>(Vec<[u8; S]>);

struct RootMeta<const S: usize> {
    page_size: u16,
    next: PageNo::u24,
}

impl<const S: usize> RootMeta<S> {
    fn from(byte: &[u8]) -> Option<Self> {
        let mut seeker = ByteSeeker::new(&byte);

        let page_size = seeker.next()?;
        let next = PageNo::u24(seeker.buf()?);

        if page_size != S.try_into().ok()? {
            
        }

        todo!()
    }
}
