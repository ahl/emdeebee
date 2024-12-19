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
    ptr::{null, null_mut},
};

use mdb_api::{
    sys::{mdb_walk_state_t, mdb_walker_t, WALK_DONE, WALK_NEXT},
    BigWalker,
};

type uintptr_t = c_ulong;

// generated
extern "C" fn tokio_task_walk_init(state: *mut mdb_walk_state_t) -> c_int {
    let me: TokioTaskWalker = TokioTaskWalker::default();
    let walk_data = Box::into_raw(Box::new(Box::new(me) as Box<dyn Walker>));

    unsafe { (*state).walk_data = walk_data.cast() };

    // make the TokioTaskWalker
    // Box<dyn Something>
    // store that in state.walk_data

    unsafe { (*state).walk_addr = 777 };

    WALK_NEXT
}

// applies to all
extern "C" fn global_step(state: *mut mdb_walk_state_t) -> c_int {
    let walk_data = unsafe { (*state).walk_data };

    let me: *mut Box<dyn Walker> = walk_data.cast();
    let ret = unsafe { &mut **me }.step();
    // let ret = unsafe { (*xxx).step() };

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
pub struct TokioTaskWalker {
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

impl BigWalker for TokioTaskWalker {
    fn linkage(&self) -> mdb_walker_t {
        TOKIO_TASK_WALKER
    }
}

pub const TOKIO_TASK_WALKER: mdb_walker_t = mdb_walker_t {
    walk_name: b"tokio_wask\0".as_ptr() as _,
    walk_descr: b"nfw is this going to work\0".as_ptr() as _,
    walk_init: Some(tokio_task_walk_init),
    walk_step: Some(global_step),
    walk_fini: Some(global_fini),
    walk_init_arg: null_mut(),
};
