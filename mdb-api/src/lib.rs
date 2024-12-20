mod alloc;
pub mod api;
pub mod dcmd;
pub mod sys;
pub mod walk;

pub use mdb_derive::Walker;

use std::{
    marker::PhantomData,
    ptr::{null, null_mut},
};

pub use sys::mdb_modinfo_t;
use sys::{mdb_dcmd_t, mdb_walker_t, MDB_API_VERSION};

pub use dcmd::Dcmd;
pub use walk::WalkStep;
pub use walk::Walker;

/// An address used in MDB.
#[derive(Clone, Copy, Debug)]
pub struct Addr(sys::uintptr_t);

impl Addr {
    /// Construct a new address.
    pub const fn new(x: u64) -> Self {
        Self(x)
    }

    /// Return this as a uintptr_t.
    pub const fn as_ptr(&self) -> sys::uintptr_t {
        self.0
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Addr(value)
    }
}

/// Information describing a loadable MDB dmod.
#[derive(Default)]
pub struct Modinfo {
    // The dcmds the module implements.
    dcmds: Vec<Box<dyn InternalLinkage<mdb_dcmd_t>>>,
    // The walkers the mdoule implements.
    walkers: Vec<Box<dyn InternalLinkage<mdb_walker_t>>>,
}

impl Modinfo {
    /// Add a dcmd to the module.
    pub fn with_dcmd<T: DcmdLinkage + 'static>(mut self) -> Self {
        self.dcmds.push(Box::new(LinkageHolder::<T>(PhantomData)));
        self
    }

    /// Add a walker to the module.
    pub fn with_walker<T: WalkerLinkage + 'static>(mut self) -> Self {
        self.walkers.push(Box::new(LinkageHolder::<T>(PhantomData)));
        self
    }
}

// Internal type for delegating to static method for generating the right
// module information for dcmds or walkers.
struct LinkageHolder<T>(PhantomData<T>);

impl<T: WalkerLinkage> InternalLinkage<mdb_walker_t> for LinkageHolder<T> {
    fn linkage(&self) -> mdb_walker_t {
        T::linkage()
    }
}

impl<T: DcmdLinkage> InternalLinkage<mdb_dcmd_t> for LinkageHolder<T> {
    fn linkage(&self) -> mdb_dcmd_t {
        T::linkage()
    }
}

// Internal trait that lets us make an object-safe type for constructing the
// right mdb info structs for dcmds or walkers.
trait InternalLinkage<T> {
    fn linkage(&self) -> T;
}

// TODO impl this when deriving `mdb::Dcmd` or whatever
pub trait DcmdLinkage {
    fn linkage() -> mdb_dcmd_t;
}

// TODO impl this when deriving `mdb::Walker` or whatever
pub trait WalkerLinkage {
    fn linkage() -> mdb_walker_t;
}

// An empty dcmd, used to terminate the list of registered commands in the
// module.
const NULL_DCMD: mdb_dcmd_t = mdb_dcmd_t {
    dc_name: null(),
    dc_usage: null(),
    dc_descr: null(),
    dc_funcp: None,
    dc_help: None,
    dc_tabp: None,
};

// An empty walkter, used to terminate the list of registered walkers in the
// module.
const NULL_WALKER: mdb_walker_t = mdb_walker_t {
    walk_name: null(),
    walk_descr: null(),
    walk_init: None,
    walk_step: None,
    walk_fini: None,
    walk_init_arg: null_mut(),
};

// Convert an array of module pieces into a null-terminated list.
fn to_null_or_native<T, N>(list: &[Box<T>]) -> *const N
where
    T: InternalLinkage<N> + ?Sized,
    N: NullNative,
{
    let mut list = list.iter().map(|item| item.linkage()).collect::<Vec<_>>();
    if list.is_empty() {
        // Return a pointer to a null item (not a null pointer).
        Box::into_raw(Box::new(N::null()))
    } else {
        // Add a null item, then pass the pointer to mdb, forgetting the vec to
        // avoid dropping it.
        list.push(N::null());
        let ptr = list.as_ptr();
        std::mem::forget(list);
        ptr
    }
}

// Trait for returning an empty item, for terminating lists.
trait NullNative {
    fn null() -> Self;
}

impl NullNative for mdb_dcmd_t {
    fn null() -> Self {
        NULL_DCMD
    }
}

impl NullNative for mdb_walker_t {
    fn null() -> Self {
        NULL_WALKER
    }
}

impl Modinfo {
    /// Create the info used to register this dmod with MDB.
    pub fn to_native(&self) -> *const mdb_modinfo_t {
        let mi_dcmds = to_null_or_native(&self.dcmds);
        let mi_walkers = to_null_or_native(&self.walkers);

        let ret = mdb_modinfo_t {
            mi_dvers: MDB_API_VERSION,
            mi_dcmds,
            mi_walkers,
        };
        Box::into_raw(Box::new(ret)).cast()
    }
}

#[macro_export]
macro_rules! mdb_print {
    ($msg:expr) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\0") };
        let arg = ::std::ffi::CString::new($msg.to_string())
            .expect("mdb_print CString::new()");
        unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    };
    ($fmt:expr, $($arg:tt)*) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\0") };
        let arg = ::std::ffi::CString::new(format!($fmt, $($arg)*))
            .expect("mdb_print CString::new()");
        unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    }
}

#[macro_export]
macro_rules! mdb_println {
    () => {
        {
            let fmt =
                unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") };
            unsafe { $crate::sys::mdb_printf(fmt.as_ptr()) };
        }
    };
    ($msg:expr) => {
        {
            let fmt =
                unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
            let arg = ::std::ffi::CString::new($msg.to_string())
                .expect("mdb_println CString::new()");
            unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            let fmt =
                unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
            let arg = ::std::ffi::CString::new(format!($fmt, $($arg)*))
                .expect("mdb_println CString::new()");
            unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
        }
    }
}
