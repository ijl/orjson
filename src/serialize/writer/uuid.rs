// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright The Rust Project Developers (2013-2014), The Uuid Project Developers (2018)

use crate::ffi::PyUuidRef;

pub(crate) fn format_hyphenated(ob: PyUuidRef, dst: &mut [u8; 36]) {
    const LOWER: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e',
        b'f',
    ];
    const GROUPS: [(usize, usize); 5] = [(0, 8), (9, 13), (14, 18), (19, 23), (24, 36)];

    let mut src: [u8; 16] = [0; 16];
    ob.value(&mut src);

    let mut group_idx = 0;
    let mut i = 0;
    while group_idx < 5 {
        let (start, end) = GROUPS[group_idx];
        let mut j = start;
        while j < end {
            let x = src[i];
            i += 1;

            dst[j] = LOWER[(x >> 4) as usize];
            dst[j + 1] = LOWER[(x & 0x0f) as usize];
            j += 2;
        }
        if group_idx < 4 {
            dst[end] = b'-';
        }
        group_idx += 1;
    }
}
