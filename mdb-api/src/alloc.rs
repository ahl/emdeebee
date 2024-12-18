use std::{
    alloc::{handle_alloc_error, GlobalAlloc, Layout},
    ffi::c_void,
};

use crate::sys::*;

#[global_allocator]
static MDB_ALLOCATOR: MdbAllocator = MdbAllocator;

struct MdbAllocator;

unsafe impl GlobalAlloc for MdbAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = mdb_alloc(layout.size() as size_t, UM_SLEEP) as *mut u8;
        if (ret as usize) & (layout.align() - 1) != 0 {
            handle_alloc_error(layout);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        mdb_free(ptr as *mut c_void, layout.size() as size_t);
    }
}
