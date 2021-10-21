use crate::utill::{insert_within_slice, swap_slices};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Entry {
    pub id: u64,
    pub value: [u8; 8],
}

#[repr(C)]
pub struct Leaf {
    pub entrys: [Entry; 255],
    pub left_child: u16,
    pub right_child: u16,
    pub len: u8,
    pub _pad: [u8; 3],
}

impl Leaf {
    pub fn new() -> Self {
        Self {
            len: 0,
            entrys: [Default::default(); 255],
            left_child: 0,
            right_child: 0,
            _pad: [0; 3],
        }
    }

    fn binery_search_by(&self, id: u64) -> Result<usize, usize> {
        self.entrys[..(self.len as usize)].binary_search_by_key(&id, |e| e.id)
    }

    /// Duplicate Id is ignored...
    ///
    /// Note: This funtion will panic, If insetion count is greater than buf size (255)
    pub fn insert_and_sort_entry(&mut self, id: u64, value: [u8; 8]) {
        if let Err(i) = self.binery_search_by(id) {
            insert_within_slice(&mut self.entrys, i, Entry { id, value });
            self.len += 1;
        }
    }

    pub fn is_full(&self) -> bool {
        self.len == 255
    }

    pub fn split(&mut self) -> (Leaf, u64) {
        let mut right = Leaf::new();
        swap_slices(&mut self.entrys[127..], &mut right.entrys);
        let mid = right.entrys[0].id;
        (right, mid)
    }

    pub fn find_value(&self, id: u64) -> Option<[u8; 8]> {
        match self.binery_search_by(id) {
            Ok(i) => Some(self.entrys[i].value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_sort_entry() {
        let mut leaf = Leaf::new();
        let ids = [1, 0, 5, 4, 2, 6, 3];

        for id in ids {
            leaf.insert_and_sort_entry(id, [0; 8]);
        }

        let sorted_ids: Vec<_> = leaf.entrys.iter().map(|&v| v.id).collect();
        assert!(sorted_ids.starts_with(&[0, 1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn split() {
        let mut left = Leaf::new();

        for (i, entry) in left.entrys.iter_mut().enumerate() {
            entry.id = 1 + i as u64;
        }

        let (right, mid) = left.split();
        let ids: Vec<u64> = (0..=255).collect();

        let left_ids: Vec<_> = left.entrys.iter().map(|&v| v.id).collect();
        let right_ids: Vec<_> = right.entrys.iter().map(|&v| v.id).collect();

        assert_eq!(mid, 128);

        assert!(left_ids.starts_with(&ids[1..=127]));
        assert!(right_ids.starts_with(&ids[128..=255]));

        assert!(left_ids.ends_with(&[0; 128]));
        assert!(right_ids.ends_with(&[0; 127]));
    }
}
