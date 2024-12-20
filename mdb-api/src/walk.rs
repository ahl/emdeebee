//! Walker types and traits

use std::ops::ControlFlow;

use crate::Addr;

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

/// The result of a walk step, with `Err(())` indicating the step failed.
pub type StepResult = Result<StepOutput, ()>;
