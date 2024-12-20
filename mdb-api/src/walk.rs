//! Walker types and traits

use std::{ffi::c_int, ops::ControlFlow, ptr::null};

use crate::{
    sys::{mdb_walk_state_t, WALK_DONE, WALK_ERR, WALK_NEXT},
    Addr,
};

/// A trait for an MDB walker.
pub trait Walker {}

/// Trait implementing a single step in an MDB walker.
///
/// This is the main trait a walker needs to implement. The other, `Walker`,
/// should be derived.
pub trait WalkStep {
    /// Take a walk step.
    fn step(&mut self) -> StepResult;
}

/// The output of a successful walk step.
///
/// If the step succeeded and has more work to do, then `Continue(Addr)` should
/// be returned. If the step succeeded and the walk is complete, then
/// `Break(())` should be returned.
pub type StepOutput = ControlFlow<(), Addr>;

/// The result of a walk step.
pub type StepResult = Result<StepOutput, StepFailed>;

/// Error indicating a walk step failed.
#[derive(Clone, Copy, Debug)]
pub struct StepFailed;

impl std::fmt::Display for StepFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Walk step failed")
    }
}

impl std::error::Error for StepFailed {}

/// Function used to step any supported walker.
///
/// This method is always registered as the `walk_step` method when registering a
/// new dmod. Implementors of the `Walker` trait are provided their own `self`
/// when we call their `Walker::step()` method. That's done by operating on a
/// trait object implementing `Walker`, which we store in the `walk_state_t`.
/// call their `Walker::step()` method.
///
/// # Safety
///
/// This is unsafe because it dereferences raw pointers internally.
pub unsafe extern "C" fn global_step(state: *mut mdb_walk_state_t) -> c_int {
    let walk_data = unsafe { (*state).walk_data };
    let me: *mut Box<dyn WalkStep> = walk_data.cast();
    let ret = unsafe { &mut **me }.step();

    match ret {
        Ok(StepOutput::Continue(addr)) => {
            let walk_callback = unsafe { (*state).walk_callback };
            let walk_cbdata = unsafe { (*state).walk_cbdata };
            unsafe { walk_callback(addr.as_ptr(), null(), walk_cbdata) };
            WALK_NEXT
        }
        Ok(StepOutput::Break(_)) => WALK_DONE,
        Err(_) => WALK_ERR,
    }
}

/// Function used to finalize any supported walker.
///
/// # Safety
///
/// This is unsafe because it dereferences raw pointers internally.
pub unsafe extern "C" fn global_fini(state: *mut mdb_walk_state_t) {
    // Take the trait object into a box, and let it go out of scope to free it.
    let walk_data = unsafe { (*state).walk_data };
    let _trait_obj: Box<Box<dyn WalkStep>> = unsafe { Box::from_raw(walk_data.cast()) };
}
