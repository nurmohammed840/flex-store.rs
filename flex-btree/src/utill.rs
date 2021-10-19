/// Insert a value on index. move all elemment to right (advance 1).
pub fn insert_within_slice<T: Copy>(arr: &mut [T], i: usize, value: T) {
    let last_idx = arr.len() - 1;
    arr.copy_within(i..last_idx, i + 1);
    arr[i] = value;
}

pub fn swap_slices<T: Clone>(arr: &mut [T], arr2: &mut [T]) {
    for (l, r) in arr.iter_mut().zip(arr2) {
        let t = r.clone();
        *r = l.clone();
        *l = t;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_within_slice() {
        let mut arr = [1, 2, 3];
        insert_within_slice(&mut arr, 0, 0);
        assert_eq!(arr, [0, 1, 2]);
    }

    #[test]
    fn test_swap_slices() {
        let mut arr1 = [0, 0, 0];
        let mut arr2 = [1, 1, 1];
        swap_slices(&mut arr1[..2], &mut arr2);
        assert_eq!(arr1, [1, 1, 0]);
        assert_eq!(arr2, [0, 0, 1]);
    }
}
