// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;

struct PyMemAllocator {}

#[global_allocator]
static ALLOCATOR: PyMemAllocator = PyMemAllocator {};

unsafe impl Sync for PyMemAllocator {}

unsafe impl GlobalAlloc for PyMemAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { pyo3_ffi::PyMem_Malloc(layout.size()) as *mut u8 }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { pyo3_ffi::PyMem_Free(ptr as *mut c_void) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let len = layout.size();
            let ptr = pyo3_ffi::PyMem_Malloc(len) as *mut u8;
            core::ptr::write_bytes(ptr, 0, len);
            ptr
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { pyo3_ffi::PyMem_Realloc(ptr as *mut c_void, new_size) as *mut u8 }
    }
}
