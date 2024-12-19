//! Walker types and traits

use std::ops::ControlFlow;

use crate::Addr;

/// Trait implementing an MDB walker.
pub trait Walker {
    /// Take a walk step.
    fn step(&mut self) -> StepResult;
}

/// The output of a successful walk step.
///
/// If the step succeeded and has more work to do, then `Continue(Addr)` should
/// be returned. If the step succeeded and the walk is complete, then
/// `Break(())` should be returned.
pub type StepOutput = ControlFlow<(), Addr>;

/// The result of a walk step, with `Err(())` indicating the step failed.
pub type StepResult = Result<StepOutput, ()>;
