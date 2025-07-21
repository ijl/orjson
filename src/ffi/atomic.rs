// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Atomic types for platform-dependent c_ulong.
//! See <https://github.com/PyO3/pyo3/blob/7836f150a0d0ba7d24264783ca6379dc1c61b8f4/pyo3-ffi/src/impl_/mod.rs#L1-L22>.

#[cfg(Py_GIL_DISABLED)]
mod atomic_c_ulong {
    pub struct GetAtomicCULong<const WIDTH: usize>();

    pub trait AtomicCULongType {
        type Type;
    }
    impl AtomicCULongType for GetAtomicCULong<32> {
        type Type = std::sync::atomic::AtomicU32;
    }
    impl AtomicCULongType for GetAtomicCULong<64> {
        type Type = std::sync::atomic::AtomicU64;
    }

    pub type TYPE =
        <GetAtomicCULong<{ std::mem::size_of::<std::os::raw::c_ulong>() * 8 }> as AtomicCULongType>::Type;
}

/// Typedef for an atomic integer to match the platform-dependent c_ulong type.
#[cfg(Py_GIL_DISABLED)]
#[doc(hidden)]
pub type AtomicCULong = atomic_c_ulong::TYPE;
