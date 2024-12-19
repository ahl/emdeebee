mod alloc;
pub mod sys;

use std::{
    ffi::CString,
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

pub struct Modinfo {
    // pub dcmds: Vec<Box<dyn Dcmd>>,
    pub walker: Vec<Box<dyn BigWalker>>,
}

pub trait BigWalker {
    fn linkage(&self) -> mdb_walker_t;
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

impl Modinfo {
    pub fn to_native(&self) -> *const mdb_modinfo_t {
        // let dcmds = self
        //     .dcmds
        //     .iter()
        //     .map(|dcmd| {
        //         let dc_name = CString::new(dcmd.name()).unwrap().into_raw();
        //         let dc_usage = CString::new(dcmd.usage()).unwrap().into_raw();
        //         let dc_descr = CString::new(dcmd.description()).unwrap().into_raw();
        //         mdb_dcmd_t {
        //             dc_name,
        //             dc_usage,
        //             dc_descr,
        //             dc_funcp: todo!(),
        //             dc_help: todo!(),
        //             dc_tabp: todo!(),
        //         }
        //     })
        //     .chain(std::iter::once(NULL_DCMD))
        //     .collect::<Vec<_>>();

        let walkers = self
            .walker
            .iter()
            .map(|w| w.linkage())
            .chain(std::iter::once(NULL_WALKER))
            .collect::<Vec<_>>();

        let mi_walkers = walkers.as_ptr();

        let ret = mdb_modinfo_t {
            mi_dvers: MDB_API_VERSION,
            mi_dcmds: null(),
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
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") };
        unsafe { $crate::sys::mdb_printf(fmt.as_ptr()) };
    };
    ($msg:expr) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
        let arg = ::std::ffi::CString::new($msg.to_string())
            .expect("mdb_println CString::new()");
        unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    };
    ($fmt:expr, $($arg:tt)*) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
        let arg = ::std::ffi::CString::new(format!($fmt, $($arg)*))
            .expect("mdb_println CString::new()");
        unsafe { $crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    }
}
