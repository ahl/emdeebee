//! Implement dcmds for an MDB dmod.

use std::ffi::{c_int, c_uint};

use crate::{
    sys::{mdb_dcmd_t, DCMD_ABORT, DCMD_ERR, DCMD_NEXT, DCMD_OK, DCMD_USAGE},
    Addr,
};

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
    pub const fn to_int(self) -> c_int {
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
    #[derive(Clone, Copy, Debug)]
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

// Mirrored after the walker thing, we have the derivable Dcmd trait, but then
// have people just impl the actual command.

/// An MDB dcmd.
pub trait Dcmd: DcmdFn {
    fn from_args(args: Vec<Arg>) -> Result<Self, String>
    where
        Self: Sized;

    fn linkage() -> mdb_dcmd_t;

    fn help() -> &'static str {
        ""
    }
}

/// Trait implementing the actual dcmd operation.
pub trait DcmdFn {
    fn call(&self, addr: Addr, flags: Flags) -> Code;
}

/// A generic argument to a dcmd.
#[derive(Clone, Debug)]
pub enum Arg {
    String(String),
    U64(u64),
}

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::String(s) => s.fmt(f),
            Arg::U64(x) => x.fmt(f),
        }
    }
}

impl TryFrom<Arg> for String {
    type Error = ();

    fn try_from(value: Arg) -> Result<Self, Self::Error> {
        match value {
            Arg::String(s) => Ok(s),
            Arg::U64(_) => Err(()),
        }
    }
}

impl TryFrom<&Arg> for String {
    type Error = ();

    fn try_from(value: &Arg) -> Result<Self, Self::Error> {
        match value {
            Arg::String(s) => Ok(s.clone()),
            Arg::U64(_) => Err(()),
        }
    }
}

macro_rules! impl_try_from_arg {
    ($int:ty) => {
        impl TryFrom<Arg> for $int {
            type Error = ();

            fn try_from(value: Arg) -> Result<Self, Self::Error> {
                match value {
                    Arg::U64(x) => Self::try_from(x).map_err(|_| ()),
                    Arg::String(s) => s.parse().map_err(|_| ()),
                }
            }
        }

        impl TryFrom<&Arg> for $int {
            type Error = ();

            fn try_from(value: &Arg) -> Result<Self, Self::Error> {
                match value {
                    Arg::U64(x) => Self::try_from(*x).map_err(|_| ()),
                    Arg::String(s) => s.parse().map_err(|_| ()),
                }
            }
        }
    };
}

impl_try_from_arg!(u8);
impl_try_from_arg!(u16);
impl_try_from_arg!(u32);
impl_try_from_arg!(u64);
impl_try_from_arg!(i8);
impl_try_from_arg!(i16);
impl_try_from_arg!(i32);
impl_try_from_arg!(i64);

// Generated code:
//
// - emits dcmd fn pointer which
// - converts arguments to arg enum
// - tries to parse according to struct defn
// - tries to construct Self from them, failing for non-optionals

// TODO add a derive for this on a struct, which generates the code in
// tokio-mdb/src/lib.rs:
//
// - the "linkage"
// - the actual unsafe fn that impls the command
// - all the marshalling into self
// - type checking for the fields of the struct
