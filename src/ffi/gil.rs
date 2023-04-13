pub struct SuspendGIL {
    tstate: Option<*mut pyo3_ffi::PyThreadState>,
    gil_count: i16,
}

impl SuspendGIL {
    pub fn new(can_suspend: bool) -> Self {
        if can_suspend {
            Self { tstate: None, gil_count: 1 }
        } else {
            Self { tstate: None, gil_count: -1 }
        }
    }

    pub fn restore(&self) -> Self {
        if self.gil_count == -1 {
            Self { tstate: None, gil_count: -1 }
        } else {
            if let Some(tstate) = self.tstate {
                unsafe { pyo3_ffi::PyEval_RestoreThread(tstate) };
            }
            Self { tstate: None, gil_count: self.gil_count + 1 }
        }
    }

    pub fn release(&self) -> Self {
        if self.gil_count == -1 {
            Self { tstate: None, gil_count: -1 }
        } else if self.gil_count == 1 {
            unsafe {
                let tstate = pyo3_ffi::PyEval_SaveThread();
                Self { tstate: Some(tstate), gil_count: 0 }
            }
        } else {
            Self { tstate: None, gil_count: self.gil_count - 1 }
        }
    }
}
