#![allow(non_camel_case_types)]
// ::walk tasks | ::task --stacks
//!  ::walk task
//! 123
//! 456
//! 678
//!
//! > 1234::task --stacks
//!

use std::{
    ffi::{c_int, c_ulong},
    ptr::null,
};

use mdb_api::sys::{mdb_walk_state_t, WALK_DONE, WALK_NEXT};

type uintptr_t = c_ulong;

// generated
extern "C" fn tokio_task_walk_init(state: *mut mdb_walk_state_t) -> c_int {
    let me: TokioTaskWalker = TokioTaskWalker::default();
    let xxx: Box<dyn Walker> = Box::new(me);
    let xxx = Box::new(xxx);
    let xxx = Box::into_raw(xxx);

    unsafe { (*state).walk_data = xxx.cast() };

    // make the TokioTaskWalker
    // Box<dyn Something>
    // store that in state.walk_data

    0
}

// applies to all
extern "C" fn global_step(state: *mut mdb_walk_state_t) -> c_int {
    let walk_data = unsafe { (*state).walk_data };

    let xxx: *mut Box<dyn Walker> = walk_data.cast();
    let ret = unsafe { (*xxx).step() };

    let addr = match ret {
        Ok(addr) => addr,
        Err(ret) => return ret,
    };

    let walk_callback = unsafe { (*state).walk_callback };
    let walk_cbdata = unsafe { (*state).walk_cbdata };

    unsafe { walk_callback(addr, null(), walk_cbdata) };

    WALK_NEXT
}
extern "C" fn global_fini(state: *mut mdb_walk_state_t) {
    let walk_data = unsafe { (*state).walk_data };
    let cheese: Box<Box<dyn Walker>> = unsafe { Box::from_raw(walk_data.cast()) };
    drop(cheese);
}

// #[derive(mdb::Walker)]
/// Description
struct TokioTaskWalker {
    current: u32,
    end: u32,
}

impl Default for TokioTaskWalker {
    fn default() -> Self {
        Self {
            current: 5,
            end: 12,
        }
    }
}

trait Walker {
    fn step(&mut self) -> Result<uintptr_t, c_int>;
}

impl Walker for TokioTaskWalker {
    fn step(&mut self) -> Result<uintptr_t, c_int> {
        if self.current <= self.end {
            Err(WALK_DONE)
        } else {
            self.current += 1;
            Ok(self.current as uintptr_t)
        }
    }
}
