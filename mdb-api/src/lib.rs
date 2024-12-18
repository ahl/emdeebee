mod alloc;
mod sys;

pub use sys::mdb_modinfo_t;

pub struct MdbDcmd {
    pub name: &'static str,
    pub usage: &'static str,
    pub description: &'static str,
    pub r#impl: T,
}

pub trait MdbDcmdImpl {
    fn name() -> String;
    fn usage() -> String;
    fn description() -> String;
    fn command();
    fn help() {}
    fn tab_complete() {}
}

pub struct MdbModInfo {
    pub dcmds: &'static [MdbDcmd],
}

impl MdbModInfo {
    pub fn to_native(&self) -> *const mdb_modinfo_t {
        todo!()
    }
}

#[macro_export]
macro_rules! mdb_print {
    ($msg:expr) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\0") };
        let arg = ::std::ffi::CString::new($msg.to_string())
            .expect("mdb_print CString::new()");
        unsafe { ::$crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    };
    ($fmt:expr, $($arg:tt)*) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\0") };
        let arg = ::std::ffi::CString::new(format!($fmt, $($arg)*))
            .expect("mdb_print CString::new()");
        unsafe { ::$crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    }
}

#[macro_export]
macro_rules! mdb_println {
    () => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") };
        unsafe { ::$crate::sys::mdb_printf(fmt.as_ptr()) };
    };
    ($msg:expr) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
        let arg = ::std::ffi::CString::new($msg.to_string())
            .expect("mdb_println CString::new()");
        unsafe { ::$crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    };
    ($fmt:expr, $($arg:tt)*) => {
        let fmt =
            unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"%s\n\0") };
        let arg = ::std::ffi::CString::new(format!($fmt, $($arg)*))
            .expect("mdb_println CString::new()");
        unsafe { ::$crate::sys::mdb_printf(fmt.as_ptr(), arg.as_ptr()) };
    }
}
