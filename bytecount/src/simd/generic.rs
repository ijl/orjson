extern crate packed_simd;

#[cfg(not(feature = "runtime-dispatch-simd"))]
use core::mem;
#[cfg(feature = "runtime-dispatch-simd")]
use std::mem;

use self::packed_simd::{u8x32, u8x64, FromCast};

const MASK: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

unsafe fn u8x64_from_offset(slice: &[u8], offset: usize) -> u8x64 {
    u8x64::from_slice_unaligned_unchecked(slice.get_unchecked(offset..))
}
unsafe fn u8x32_from_offset(slice: &[u8], offset: usize) -> u8x32 {
    u8x32::from_slice_unaligned_unchecked(slice.get_unchecked(offset..))
}

fn sum_x64(u8s: &u8x64) -> usize {
    let mut store = [0; mem::size_of::<u8x64>()];
    u8s.write_to_slice_unaligned(&mut store);
    store.iter().map(|&e| e as usize).sum()
}
fn sum_x32(u8s: &u8x32) -> usize {
    let mut store = [0; mem::size_of::<u8x32>()];
    u8s.write_to_slice_unaligned(&mut store);
    store.iter().map(|&e| e as usize).sum()
}

pub fn chunk_count(haystack: &[u8], needle: u8) -> usize {
    assert!(haystack.len() >= 32);

    unsafe {
        let mut offset = 0;
        let mut count = 0;

        let needles_x64 = u8x64::splat(needle);

        // 16320
        while haystack.len() >= offset + 64 * 255 {
            let mut counts = u8x64::splat(0);
            for _ in 0..255 {
                counts -= u8x64::from_cast(u8x64_from_offset(haystack, offset).eq(needles_x64));
                offset += 64;
            }
            count += sum_x64(&counts);
        }

        // 8192
        if haystack.len() >= offset + 64 * 128 {
            let mut counts = u8x64::splat(0);
            for _ in 0..128 {
                counts -= u8x64::from_cast(u8x64_from_offset(haystack, offset).eq(needles_x64));
                offset += 64;
            }
            count += sum_x64(&counts);
        }

        let needles_x32 = u8x32::splat(needle);

        // 32
        let mut counts = u8x32::splat(0);
        for i in 0..(haystack.len() - offset) / 32 {
            counts -= u8x32::from_cast(u8x32_from_offset(haystack, offset + i * 32).eq(needles_x32));
        }
        if haystack.len() % 32 != 0 {
            counts -= u8x32::from_cast(u8x32_from_offset(haystack, haystack.len() - 32).eq(needles_x32)) &
                      u8x32_from_offset(&MASK, haystack.len() % 32);
        }
        count += sum_x32(&counts);

        count
    }
}

fn is_leading_utf8_byte_x64(u8s: u8x64) -> u8x64 {
    u8x64::from_cast((u8s & u8x64::splat(0b1100_0000)).ne(u8x64::splat(0b1000_0000)))
}

fn is_leading_utf8_byte_x32(u8s: u8x32) -> u8x32 {
    u8x32::from_cast((u8s & u8x32::splat(0b1100_0000)).ne(u8x32::splat(0b1000_0000)))
}

pub fn chunk_num_chars(utf8_chars: &[u8]) -> usize {
    assert!(utf8_chars.len() >= 32);

    unsafe {
        let mut offset = 0;
        let mut count = 0;

        // 16320
        while utf8_chars.len() >= offset + 64 * 255 {
            let mut counts = u8x64::splat(0);
            for _ in 0..255 {
                counts -= is_leading_utf8_byte_x64(u8x64_from_offset(utf8_chars, offset));
                offset += 64;
            }
            count += sum_x64(&counts);
        }

        // 8192
        if utf8_chars.len() >= offset + 64 * 128 {
            let mut counts = u8x64::splat(0);
            for _ in 0..128 {
                counts -= is_leading_utf8_byte_x64(u8x64_from_offset(utf8_chars, offset));
                offset += 64;
            }
            count += sum_x64(&counts);
        }

        // 32
        let mut counts = u8x32::splat(0);
        for i in 0..(utf8_chars.len() - offset) / 32 {
            counts -= is_leading_utf8_byte_x32(u8x32_from_offset(utf8_chars, offset + i * 32));
        }
        if utf8_chars.len() % 32 != 0 {
            counts -= is_leading_utf8_byte_x32(u8x32_from_offset(utf8_chars, utf8_chars.len() - 32)) &
                      u8x32_from_offset(&MASK, utf8_chars.len() % 32);
        }
        count += sum_x32(&counts);

        count
    }
}
