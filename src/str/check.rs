// SPDX-License-Identifier: (Apache-2.0 OR MIT)

const STRIDE_SIZE: usize = 8;

pub fn is_four_byte(buf: &str) -> bool {
    let as_bytes = buf.as_bytes();
    let len = as_bytes.len();
    unsafe {
        let mut idx = 0;
        while idx < len.saturating_sub(STRIDE_SIZE) {
            let mut val: bool = false;
            val |= *as_bytes.get_unchecked(idx) > 239;
            val |= *as_bytes.get_unchecked(idx + 1) > 239;
            val |= *as_bytes.get_unchecked(idx + 2) > 239;
            val |= *as_bytes.get_unchecked(idx + 3) > 239;
            val |= *as_bytes.get_unchecked(idx + 4) > 239;
            val |= *as_bytes.get_unchecked(idx + 5) > 239;
            val |= *as_bytes.get_unchecked(idx + 6) > 239;
            val |= *as_bytes.get_unchecked(idx + 7) > 239;
            idx += STRIDE_SIZE;
            if val {
                return true;
            }
        }
        let mut ret = false;
        while idx < len {
            ret |= *as_bytes.get_unchecked(idx) > 239;
            idx += 1;
        }
        ret
    }
}
