use std::cell::RefCell;
use std::rc::Rc;

struct State {
    thread_state: Option<*mut pyo3_ffi::PyThreadState>,
    gil_count: i16,
}

pub struct ReleasedGIL {
    state: Option<Rc<RefCell<State>>>,
}

pub struct LockedGIL<'a> {
    locked_state: &'a Rc<RefCell<State>>,
}

impl<'a> Drop for LockedGIL<'a> {
    fn drop(&mut self) {
        ReleasedGIL::do_unlock(self.locked_state);
    }
}

impl ReleasedGIL {
    pub fn new_unlocked() -> Self {
        let thread_state = unsafe { pyo3_ffi::PyEval_SaveThread() };
        let state = State {
            thread_state: Some(thread_state),
            gil_count: 0,
        };
        Self {
            state: Some(Rc::new(RefCell::new(state))),
        }
    }

    pub fn new_dummy() -> Self {
        Self { state: None }
    }

    #[inline]
    pub fn gil_locked(&self) -> Option<LockedGIL> {
        if let Some(state) = &self.state {
            ReleasedGIL::lock(state);
            Some(LockedGIL {
                locked_state: state,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn is_released(&self) -> bool {
        self.state
            .as_ref()
            .map_or(false, |s| s.borrow().thread_state.is_some())
    }

    fn lock(state: &Rc<RefCell<State>>) {
        state.replace_with(|s| {
            if let Some(thread_state) = s.thread_state {
                unsafe { pyo3_ffi::PyEval_RestoreThread(thread_state) };
            }
            State {
                thread_state: None,
                gil_count: s.gil_count + 1,
            }
        });
    }

    fn do_unlock(state: &Rc<RefCell<State>>) {
        state.replace_with(|s| {
            if s.gil_count == 1 {
                let thread_state = unsafe { pyo3_ffi::PyEval_SaveThread() };
                State {
                    thread_state: Some(thread_state),
                    gil_count: 0,
                }
            } else {
                State {
                    thread_state: None,
                    gil_count: s.gil_count - 1,
                }
            }
        });
    }
}

impl Drop for ReleasedGIL {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            ReleasedGIL::lock(state);
        }
    }
}
