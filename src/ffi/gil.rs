use std::cell::RefCell;

struct InternalState {
    py_thread_state: Option<*mut pyo3_ffi::PyThreadState>,
    lock_counter: i16,
}

pub struct GIL {
    state: Option<RefCell<InternalState>>,
}

pub struct GuardedLock<'a> {
    guarded_state: &'a RefCell<InternalState>,
}

impl GIL {
    pub fn new_released() -> Self {
        let py_thread_state = unsafe { pyo3_ffi::PyEval_SaveThread() };
        let state = InternalState {
            py_thread_state: Some(py_thread_state),
            lock_counter: 0,
        };
        Self {
            state: Some(RefCell::new(state)),
        }
    }

    pub fn new_unreleasable() -> Self {
        Self { state: None }
    }

    #[inline]
    pub fn gil_locked(&self) -> Option<GuardedLock> {
        if let Some(state) = &self.state {
            GIL::do_lock(state);
            Some(GuardedLock {
                guarded_state: state,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn is_released(&self) -> bool {
        self.state
            .as_ref()
            .map_or(false, |s| s.borrow().py_thread_state.is_some())
    }

    fn do_lock(state: &RefCell<InternalState>) {
        state.replace_with(|s| {
            if let Some(py_thread_state) = s.py_thread_state {
                unsafe { pyo3_ffi::PyEval_RestoreThread(py_thread_state) };
            }
            InternalState {
                py_thread_state: None,
                lock_counter: s.lock_counter + 1,
            }
        });
    }

    fn do_release(state: &RefCell<InternalState>) {
        state.replace_with(|s| {
            if s.lock_counter == 1 {
                let py_thread_state = unsafe { pyo3_ffi::PyEval_SaveThread() };
                InternalState {
                    py_thread_state: Some(py_thread_state),
                    lock_counter: 0,
                }
            } else {
                InternalState {
                    py_thread_state: None,
                    lock_counter: s.lock_counter - 1,
                }
            }
        });
    }
}

impl<'a> Drop for GuardedLock<'a> {
    fn drop(&mut self) {
        GIL::do_release(self.guarded_state);
    }
}

impl Drop for GIL {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            GIL::do_lock(state);
        }
    }
}
