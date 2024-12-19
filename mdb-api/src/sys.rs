#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_uint, c_ulong, c_ulonglong, c_ushort, c_void};

pub(crate) type size_t = c_ulong;
pub(crate) type uintptr_t = c_ulong;
pub(crate) type uintmax_t = c_ulonglong;

pub const MDB_API_VERSION: c_ushort = 5;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct mdb_object_t {
    pub obj_name: *const c_char,
    pub obj_fullname: *const c_char,
    pub obj_base: uintptr_t,
    pub obj_size: uintptr_t,
}

pub type mdb_object_cb_t = unsafe extern "C" fn(*const mdb_object_t, *mut c_void) -> c_int;

// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// pub struct mdb_symbol_t {
//     pub sym_name: *const c_char,
//     pub sym_object: *const c_char,
//     pub sym_sym: *const GElf_Sym,
//     sym_table: c_uint,
//     sym_id: c_uint,
// }

// pub type mdb_symbol_cb_t = unsafe extern "C" fn(*const mdb_symbol_t, *mut c_void) -> c_int;

pub enum mdb_tab_cookie_t {}

pub type mdb_dcmd_f = unsafe extern "C" fn(uintptr_t, c_uint, c_int, *const mdb_arg_t) -> c_int;

/*
 * Command function return codes:
 */
pub const DCMD_OK: c_int = 0;
pub const DCMD_ERR: c_int = 1;
pub const DCMD_USAGE: c_int = 2;
pub const DCMD_NEXT: c_int = 3;
pub const DCMD_ABORT: c_int = 4;

// Dcmd flags
pub const DCMD_ADDRSPEC: c_uint = 0x01;
pub const DCMD_LOOP: c_uint = 0x02;
pub const DCMD_LOOPFIRST: c_uint = 0x04;
pub const DCMD_PIPE: c_uint = 0x08;
pub const DCMD_PIPE_OUT: c_uint = 0x10;

type mdb_dcmd_tab_f =
    unsafe extern "C" fn(*const mdb_tab_cookie_t, c_uint, c_int, *const mdb_arg_t);

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum mdb_type_t {
    MDB_TYPE_STRING = 0,
    MDB_TYPE_IMMEDIATE = 1,
    MDB_TYPE_CHAR = 2,
}

#[repr(C)]
pub union mdb_arg_union {
    pub a_str: *const c_char,
    pub a_val: uintmax_t,
    pub a_char: c_char,
}

#[repr(C)]
pub struct mdb_arg_t {
    pub a_type: mdb_type_t,
    pub a_un: mdb_arg_union,
}

#[repr(C)]
pub struct mdb_dcmd_t {
    pub dc_name: *const c_char,
    pub dc_usage: *const c_char,
    pub dc_descr: *const c_char,
    pub dc_funcp: Option<mdb_dcmd_f>,
    pub dc_help: Option<unsafe extern "C" fn()>,
    pub dc_tabp: Option<mdb_dcmd_tab_f>,
}

impl mdb_dcmd_t {
    pub const fn null() -> mdb_dcmd_t {
        mdb_dcmd_t {
            dc_name: std::ptr::null(),
            dc_usage: std::ptr::null(),
            dc_descr: std::ptr::null(),
            dc_funcp: None,
            dc_help: None,
            dc_tabp: None,
        }
    }
}

pub type mdb_walk_cb_t = unsafe extern "C" fn(uintptr_t, *const c_void, *mut c_void) -> c_int;

pub const WALK_ERR: c_int = -1;
pub const WALK_NEXT: c_int = 0;
pub const WALK_DONE: c_int = 1;

#[repr(C)]
pub struct mdb_walk_state_t {
    pub walk_callback: mdb_walk_cb_t,
    pub walk_cbdata: *mut c_void,
    pub walk_addr: uintptr_t,
    pub walk_data: *mut c_void,
    pub walk_arg: *mut c_void,
    pub walk_layer: *const c_void,
}

#[repr(C)]
pub struct mdb_walker_t {
    pub walk_name: *const c_char,
    pub walk_descr: *const c_char,
    pub walk_init: Option<unsafe extern "C" fn(*mut mdb_walk_state_t) -> c_int>,
    pub walk_step: Option<unsafe extern "C" fn(*mut mdb_walk_state_t) -> c_int>,
    pub walk_fini: Option<unsafe extern "C" fn(*mut mdb_walk_state_t)>,
    pub walk_init_arg: *mut c_void,
}

#[repr(C)]
pub struct mdb_modinfo_t {
    pub mi_dvers: c_ushort,
    pub mi_dcmds: *const mdb_dcmd_t,
    pub mi_walkers: *const mdb_walker_t,
}

pub const UM_NOSLEEP: c_uint = 0x0;
pub const UM_SLEEP: c_uint = 0x1;

extern "C" {
    pub fn mdb_printf(fmt: *const c_char, ...);

    pub fn mdb_alloc(size: size_t, flags: c_uint) -> *mut c_void;
    pub fn mdb_free(void: *mut c_void, size: size_t);
}
