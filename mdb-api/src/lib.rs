mod alloc;
pub mod sys;

use std::{
    marker::PhantomData,
    ptr::{null, null_mut},
};

pub use sys::mdb_modinfo_t;
use sys::{mdb_dcmd_t, mdb_walker_t, MDB_API_VERSION};

pub trait Dcmd {
    fn name(&self) -> String;
    fn usage(&self) -> String;
    fn description(&self) -> String;
    fn command();
}

#[derive(Default)]
pub struct Modinfo {
    dcmds: Vec<Box<dyn InternalLinkage<mdb_dcmd_t>>>,
    walkers: Vec<Box<dyn InternalLinkage<mdb_walker_t>>>,
}

impl Modinfo {
    pub fn with_dcmd<T: DcmdLinkage + 'static>(mut self) -> Self {
        self.dcmds.push(Box::new(LinkageHolder::<T>(PhantomData)));
        self
    }
    pub fn with_walker<T: WalkerLinkage + 'static>(mut self) -> Self {
        self.walkers.push(Box::new(LinkageHolder::<T>(PhantomData)));
        self
    }
}

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

trait InternalLinkage<T> {
    fn linkage(&self) -> T;
}

pub trait DcmdLinkage {
    fn linkage() -> mdb_dcmd_t;
}
pub trait WalkerLinkage {
    fn linkage() -> mdb_walker_t;
}

const NULL_DCMD: mdb_dcmd_t = mdb_dcmd_t {
    dc_name: null(),
    dc_usage: null(),
    dc_descr: null(),
    dc_funcp: None,
    dc_help: None,
    dc_tabp: None,
};

const NULL_WALKER: mdb_walker_t = mdb_walker_t {
    walk_name: null(),
    walk_descr: null(),
    walk_init: None,
    walk_step: None,
    walk_fini: None,
    walk_init_arg: null_mut(),
};

fn to_null_or_native<T, N>(list: &[Box<T>]) -> *const N
where
    T: InternalLinkage<N> + ?Sized,
    N: NullNative,
{
    let mut list = list.iter().map(|item| item.linkage()).collect::<Vec<_>>();
    if list.is_empty() {
        null()
    } else {
        list.push(N::null());
        list.as_ptr()
    }
}

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
