unsafe fn copy_without_overlap<T: Clone>(slice: &mut [T], src: usize, dest: usize, count: usize) {
    let start = slice.as_mut_ptr();
    let src_slice = core::slice::from_raw_parts_mut(start.add(src), count);
    let dest_slice = core::slice::from_raw_parts_mut(start.add(dest), count);
    dest_slice.clone_from_slice(src_slice);
}

pub unsafe fn copy_within_slice<T: Clone>(slice: &mut [T], src: usize, dest: usize, count: usize) {
    if count == 0 {
        return;
    }
    if dest < src {
        let diff = src - dest;
        if diff >= count {
            copy_without_overlap(slice, src, dest, count);
            return;
        }
        let mut remaining = count;
        let mut copied = 0;
        while copied < count {
            let to_copy = diff.min(remaining);
            let chunk_src = src + copied;
            let chunk_dest = dest + copied;
            copy_without_overlap(slice, chunk_src, chunk_dest, to_copy);
            remaining -= to_copy;
            copied += to_copy;
        }
    } else if dest > src {
        let diff = dest - src;
        if diff >= count {
            copy_without_overlap(slice, src, dest, count);
            return;
        }
        let mut remaining = count;
        let mut copied = 0;
        while copied < count {
            let to_copy = diff.min(remaining);
            let chunk_src = src + count - copied - to_copy;
            let chunk_dest = dest + count - copied - to_copy;
            copy_without_overlap(slice, chunk_src, chunk_dest, to_copy);
            remaining -= to_copy;
            copied += to_copy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_forward_no_overlap() {
        let mut data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        unsafe { copy_within_slice(&mut data, 1, 5, 3) };
        let expected = vec![1, 2, 3, 4, 5, 2, 3, 4, 9];
        assert_eq!(data, expected);
    }

    #[test]
    fn copy_backward_no_overlap() {
        let mut data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        unsafe { copy_within_slice(&mut data, 5, 1, 3) };
        let expected = vec![1, 6, 7, 8, 5, 6, 7, 8, 9];
        assert_eq!(data, expected);
    }

    #[test]
    fn copy_backward_with_overlap() {
        let mut data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        unsafe { copy_within_slice(&mut data, 3, 1, 5) };
        let expected = vec![1, 4, 5, 6, 7, 8, 7, 8, 9];
        assert_eq!(data, expected);
    }

    #[test]
    fn copy_forward_with_overlap() {
        let mut data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        unsafe { copy_within_slice(&mut data, 1, 3, 5) };
        let expected = vec![1, 2, 3, 2, 3, 4, 5, 6, 9];
        assert_eq!(data, expected);
    }
}
