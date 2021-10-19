use crate::utill::swap_slices;

pub struct Branch {
    pub ids: [u64; 408],
    pub childs: [u16; 409],
}

impl Branch {
    pub fn create_root(id: u64, left_child: u16, right_child: u16) -> Self {
        let mut branch = Branch::new();
        branch.ids[0] = id;
        branch.childs[0] = left_child;
        branch.childs[1] = right_child;
        branch
    }

    pub fn new() -> Self {
        Self {
            ids: [0; 408],
            childs: [0; 409],
        }
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

    pub fn update(&mut self, value: (u64, u16)) {
        // insert_within_slice(&mut self.ids, mid, page_no);
        // insert_within_slice(&mut self.childs, p, page_no + 1);
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
