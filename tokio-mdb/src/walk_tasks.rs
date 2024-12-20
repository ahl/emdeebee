#![allow(non_camel_case_types)]
// ::walk tasks | ::task --stacks
//!  ::walk task
//! 123
//! 456
//! 678
//!
//! > 1234::task --stacks
//!

use mdb_api::{
    walk::{StepOutput, StepResult},
    Addr, WalkStep,
};

/// Walk Tokio tasks
#[derive(mdb_api::Walker)]
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

impl WalkStep for TokioTaskWalker {
    fn step(&mut self) -> StepResult {
        if self.current >= self.end {
            Ok(StepOutput::Break(()))
        } else {
            self.current += 1;
            Ok(StepOutput::Continue(Addr::new(self.current as _)))
        }
    }
}
