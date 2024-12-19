#![allow(non_camel_case_types)]

//! API to `mdb_*` library functions that modules can use.

use std::ffi::{c_char, c_void};

use crate::{
    sys::{size_t, uintptr_t},
    Addr,
};

// NOTE: core::ffi::c_ssize_t is unstable.
type ssize_t = isize;

extern "C" {
    fn mdb_vread(buf: *mut c_void, len: size_t, addr: uintptr_t) -> ssize_t;
    fn mdb_vwrite(buf: *const c_void, len: size_t, addr: uintptr_t) -> ssize_t;
    fn mdb_readstr(buf: *const c_char, len: size_t, addr: uintptr_t) -> ssize_t;
}

/// Read memory from the target virtual address space.
///
/// If the read is successful, return `Some(_)` with the contained data.
/// Otherwise, return None.
pub fn vread(addr: Addr, len: usize) -> Option<Vec<u8>> {
    let len_ = u64::try_from(len).ok()?;
    let mut buf = vec![0; len];
    let ret = unsafe { mdb_vread(buf.as_mut_ptr().cast(), len_, addr.as_ptr()) };
    if ret == -1 {
        None
    } else {
        Some(buf)
    }
}

/// Write memory to the target virtual address space.
///
/// Return the number of bytes written on success, or None on failure.
pub fn vwrite(buf: &[u8], addr: Addr) -> Option<usize> {
    let len_ = u64::try_from(buf.len()).ok()?;
    let ret = unsafe { mdb_vwrite(buf.as_ptr().cast(), len_, addr.as_ptr()) };
    if ret == -1 {
        None
    } else {
        usize::try_from(ret).ok()
    }
}

/// Read a string from the specified virtual address, up the specified len.
///
/// Return the string on success, or None on failure. None is also returned if
/// the string isn't valid UTF-8.
pub fn readstr(addr: Addr, len: usize) -> Option<String> {
    let len_ = u64::try_from(len).ok()?;
    let mut buf = vec![0; len];
    let ret = unsafe { mdb_readstr(buf.as_mut_ptr().cast(), len_, addr.as_ptr()) };
    if ret == -1 {
        None
    } else {
        let actual_len = usize::try_from(ret).ok()?;
        let buf = if actual_len < buf.len() {
            buf[..actual_len].to_vec()
        } else {
            buf
        };
        String::from_utf8(buf).ok()
    }
}
