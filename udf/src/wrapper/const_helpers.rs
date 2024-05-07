use std::str;

/// Similar to `copy_from_slice` but works at comptime.
///
/// Takes a `start` offset so we can index into an existing slice.
macro_rules! const_arr_copy {
    ($dst:expr, $src:expr, $start:expr) => {{
        let max_idx = $dst.len() - $start;
        let (to_write, add_ellipsis) = if $src.len() <= (max_idx.saturating_sub($start)) {
            ($src.len(), false)
        } else {
            ($src.len().saturating_sub(4), true)
        };

        let mut i = 0;
        while i < to_write {
            $dst[i + $start] = $src[i];
            i += 1;
        }

        if add_ellipsis {
            while i < $dst.len() - $start {
                $dst[i + $start] = b'.';
                i += 1;
            }
        }

        i
    }};
}

macro_rules! const_write_all {
    ($dst:expr, $src_arr:expr, $start:expr) => {{
        let mut offset = $start;

        let mut i = 0;
        while i < $src_arr.len() && offset < $dst.len() {
            offset += const_arr_copy!($dst, $src_arr[i].as_bytes(), offset);
            i += 1;
        }

        offset - $start
    }};
}

pub const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }

        i += 1;
    }

    true
}

pub const fn const_slice_eq(a: &[&str], b: &[&str]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if !const_str_eq(a[i], b[i]) {
            return false;
        }

        i += 1;
    }

    true
}

pub const fn const_slice_to_str(s: &[u8], len: usize) -> &str {
    assert!(len <= s.len());
    // FIXME(msrv): use const `split_at` once our MSRV gets to 1.71
    // SAFETY: validated inbounds above
    let buf = unsafe { std::slice::from_raw_parts(s.as_ptr(), len) };

    match str::from_utf8(buf) {
        Ok(v) => v,
        Err(_e) => panic!("utf8 error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arr_copy() {
        let mut x = [0u8; 20];
        let w1 = const_arr_copy!(x, b"foobar", 0);
        let s = const_slice_to_str(x.as_slice(), w1);
        assert_eq!(s, "foobar");

        let w2 = const_arr_copy!(x, b"foobar", w1);
        let s = const_slice_to_str(x.as_slice(), w1 + w2);
        assert_eq!(s, "foobarfoobar");

        let mut x = [0u8; 6];
        let written = const_arr_copy!(x, b"foobar", 0);
        let s = const_slice_to_str(x.as_slice(), written);
        assert_eq!(s, "foobar");

        let mut x = [0u8; 5];
        let written = const_arr_copy!(x, b"foobar", 0);
        let s = const_slice_to_str(x.as_slice(), written);
        assert_eq!(s, "fo...");
    }

    #[test]
    fn test_const_write_all() {
        let mut x = [0u8; 20];
        let w1 = const_write_all!(x, ["foo", "bar", "baz"], 0);
        let s = const_slice_to_str(x.as_slice(), w1);
        assert_eq!(s, "foobarbaz");
    }
}
