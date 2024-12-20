//! Implement dcmds for an MDB dmod.

use std::ffi::{c_int, c_uint};

use crate::sys::{DCMD_ABORT, DCMD_ERR, DCMD_NEXT, DCMD_OK, DCMD_USAGE};

/// The return code from a dcmd.
#[derive(Clone, Copy, Debug)]
pub enum Code {
    /// Command completed successfully.
    Ok,
    /// Command failed.
    Err,
    /// Invalid arguments specified, automatically print usage.
    Usage,
    /// Automatically invoke the next dcmd with the same arguments.
    Next,
    /// Command failed, and the pipeline or loop should be aborted.
    Abort,
}

impl Code {
    #[allow(dead_code)]
    pub(crate) const fn to_int(self) -> c_int {
        match self {
            Code::Ok => DCMD_OK,
            Code::Err => DCMD_ERR,
            Code::Usage => DCMD_USAGE,
            Code::Next => DCMD_NEXT,
            Code::Abort => DCMD_ABORT,
        }
    }
}

bitflags::bitflags! {
    /// Flags passed to a dcmd.
    pub struct Flags: c_uint {
        /// An explicit address was specified for the dcmd.
        const AddrSpec = 0x01;
        /// The dcmd was invoked in a loop.
        const Loop = 0x02;
        /// This is the first invocation of the dcmd in a loop.
        const LoopFirst = 0x04;
        /// The dcmd was invoked with input from a pipeline.
        const Pipe = 0x08;
        /// The dcmd was invoked with ouput set to a pipeline.
        const PipeOut = 0x10;
    }
}

/// An MDB dcmd.
pub trait Dcmd {
    /// The name of the command.
    fn name(&self) -> &'static str;

    /// The long-form usage of the command.
    fn usage(&self) -> Option<&'static str> {
        None
    }

    /// The short description of the command.
    fn description(&self) -> &'static str;

    /// The function invoking the command itself.
    fn command(&self) -> Code;
}

// TODO add a derive for this on a struct, which generates the code in
// tokio-mdb/src/lib.rs:
//
// - the "linkage"
// - the actual unsafe fn that impls the command
// - all the marshalling into self
// - type checking for the fields of the struct
