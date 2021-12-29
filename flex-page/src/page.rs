use crate::{locker::Lock, page_no::PageNo};
use std::fs::File;

pub struct Page<'a, P: PageNo, const PS: usize> {
    pub no: P,
    pub lock: Lock<'a, P>,
    pub file: &'static File,
    pub buf: [u8; PS],
}

impl<P: PageNo, const PS: usize> Page<'_, P, PS> {
    // fn read() {}
}
