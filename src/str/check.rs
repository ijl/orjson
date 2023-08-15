// SPDX-License-Identifier: (Apache-2.0 OR MIT)

const STRIDE_SIZE: usize = 8;

pub fn is_four_byte(buf: &str) -> bool {
    let as_bytes = buf.as_bytes();
    let chunks = as_bytes.chunks_exact(STRIDE_SIZE);
    let remainder = chunks.remainder();
    for chunk in chunks {
        let mut val = false;
        for &b in chunk {
            val |= b > 239;
        }
        if val {
            return true;
        }
    }
    let mut val = false;
    for &b in remainder {
        val |= b > 239;
    }
    val
}
