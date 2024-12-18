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
    mdb_println,
    sys::{mdb_walk_state_t, mdb_walker_t, WALK_DONE, WALK_ERR, WALK_NEXT},
    walk::{StepOutput, StepResult},
    Addr, Walker, WalkerLinkage,
};

type uintptr_t = c_ulong;

// generated
extern "C" fn tokio_task_walk_init(state: *mut mdb_walk_state_t) -> c_int {
    mdb_println!("here");
    let me: TokioTaskWalker = TokioTaskWalker::default();
    let walk_data = Box::into_raw(Box::new(Box::new(me) as Box<dyn Walker>));

    unsafe { (*state).walk_data = walk_data.cast() };

    // make the TokioTaskWalker
    // Box<dyn Something>
    // store that in state.walk_data

    unsafe { (*state).walk_addr = 777 };

    mdb_println!("next");

    WALK_NEXT
}

// Function used to step any supported walker.
//
// This method is always registered as the `walk_step` method when registering a
// new dmod. Implementors of the `Walker` trait are provided their own `self`
// when we call their `Walker::step()` method. That's done by operating on a
// trait object implementing `Walker`, which we store in the `walk_state_t`.
// call their `Walker::step()` method.
extern "C" fn global_step(state: *mut mdb_walk_state_t) -> c_int {
    let walk_data = unsafe { (*state).walk_data };
    let me: *mut Box<dyn Walker> = walk_data.cast();
    let ret = unsafe { &mut **me }.step();

    match ret {
        Ok(StepOutput::Continue(addr)) => {
            let walk_callback = unsafe { (*state).walk_callback };
            let walk_cbdata = unsafe { (*state).walk_cbdata };
            unsafe { walk_callback(addr.as_ptr(), null(), walk_cbdata) };
            WALK_NEXT
        }
        Ok(StepOutput::Break(_)) => {
            mdb_println!("walk done");
            return WALK_DONE;
        }
        Err(_) => {
            mdb_println!("walk step failed");
            return WALK_ERR;
        }
    }
}

// Function used to finalize any supported walker.
extern "C" fn global_fini(state: *mut mdb_walk_state_t) {
    // Take the trait object into a box, and let it go out of scope to free it.
    let walk_data = unsafe { (*state).walk_data };
    let _trait_obj: Box<Box<dyn Walker>> = unsafe { Box::from_raw(walk_data.cast()) };
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

/*
trait Walker {
    fn step(&mut self) -> Result<uintptr_t, c_int>;
}
*/

impl Walker for TokioTaskWalker {
    fn step(&mut self) -> StepResult {
        if self.current >= self.end {
            Ok(StepOutput::Break(()))
            //Err(WALK_DONE)
        } else {
            self.current += 1;
            Ok(StepOutput::Continue(Addr::new(self.current as _)))
        }
    }
}

impl Drop for TokioTaskWalker {
    fn drop(&mut self) {
        mdb_println!("I have been dropped!")
    }
}

// generated
impl WalkerLinkage for TokioTaskWalker {
    fn linkage() -> mdb_walker_t {
        mdb_walker_t {
            walk_name: b"tokio_walk\0".as_ptr() as _,
            walk_descr: b"nfw is this going to work\0".as_ptr() as _,
            walk_init: Some(tokio_task_walk_init),
            walk_step: Some(global_step),
            walk_fini: Some(global_fini),
            walk_init_arg: null_mut(),
        }
    }
}
