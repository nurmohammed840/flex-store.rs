use crate::utill::{insert_within_slice, swap_slices};

#[repr(C)]
pub struct Branch {
    pub ids: [u64; 408],
    pub childs: [u16; 409],
    pub _pad: [u8; 6],
}

impl Branch {
    pub fn new() -> Self {
        Self {
            ids: [0; 408],
            childs: [0; 409],
            _pad: [0; 6],
        }
    }

    pub fn create_root(id: u64, left_child: u16, right_child: u16) -> Self {
        let mut branch = Branch::new();
        branch.ids[0] = id;
        branch.childs[0] = left_child;
        branch.childs[1] = right_child;
        branch
    }

    /// -> index
    pub fn lookup(&self, id: u64) -> usize {
        let mut i: usize = 0;
        for _id in self.ids.into_iter().filter(|&id| id != 0) {
            if id < _id {
                return i;
            }
            i += 1;
        }
        i
    }

    pub fn update(&mut self, i: usize, (mid, page_no): (u64, u16)) {
        insert_within_slice(&mut self.ids, i, mid);
        insert_within_slice(&mut self.childs, i + 1, page_no);
    }

    pub fn is_full(&self) -> bool {
        *self.ids.last().unwrap() != 0
    }

    pub fn split(&mut self) -> (Branch, u64) {
        let mut right = Branch::new();

        swap_slices(&mut self.ids[204..], &mut right.ids);
        swap_slices(&mut self.childs[204..], &mut right.childs);

        let mid = self.ids[203];
        self.ids[203] = 0;
        (right, mid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup() {
        let mut branch = Branch::new();
        branch.ids[..3].copy_from_slice(&[10, 15, 20]);
        assert_eq!(branch.lookup(0), 0);
        assert_eq!(branch.lookup(9), 0);
        assert_eq!(branch.lookup(11), 1);
        assert_eq!(branch.lookup(14), 1);
        assert_eq!(branch.lookup(16), 2);
        assert_eq!(branch.lookup(19), 2);
        assert_eq!(branch.lookup(21), 3);
        assert_eq!(branch.lookup(100), 3);

        assert_eq!(branch.lookup(10), 1);
        assert_eq!(branch.lookup(15), 2);
        assert_eq!(branch.lookup(20), 3);
    }

    #[test]
    fn split() {
        let mut left = Branch::new();

        for (i, id) in left.ids.iter_mut().enumerate() {
            *id = 1 + i as u64;
        }
        for (i, chlid) in left.childs.iter_mut().enumerate() {
            *chlid = 1 + i as u16;
        }

        let ints: Vec<_> = (0..=408).collect();
        let cints: Vec<_> = (0..=409).collect();
        let (right, mid) = left.split();

        assert_eq!(mid, 204);

        assert!(left.ids.ends_with(&[0; 205]));
        assert!(right.ids.ends_with(&[0; 204]));

        assert!(left.childs.ends_with(&[0; 205]));
        assert!(right.childs.ends_with(&[0; 204]));

        assert!(left.ids.starts_with(&ints[1..=203]));
        assert!(right.ids.starts_with(&ints[205..=408]));

        assert!(left.childs.starts_with(&cints[1..=204]));
        assert!(right.childs.starts_with(&cints[205..=409]));
    }
}
