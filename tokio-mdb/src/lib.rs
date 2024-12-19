mod walk_tasks;

use std::marker::PhantomData;

use mdb_api::{
    mdb_modinfo_t,
    sys::{mdb_dcmd_t, mdb_walk_state_t, mdb_walker_t},
    Modinfo,
};
use walk_tasks::TokioTaskWalker;

// const MODINFO: MdbModInfo = MdbModInfo {
//     dcmds: &[MdbDcmd {
//         name: "test1",
//         usage: "test1 usage",
//         description: "this is test1!",
//         command: ??
//     }],
// };

struct HelloDcmd;

// impl MdbDcmdImpl for HelloDcmd {
//     fn name() -> String {
//         todo!()
//     }

//     fn usage() -> String {
//         todo!()
//     }

//     fn description() -> String {
//         todo!()
//     }

//     // fn command(addr: u64, flags: u16, args: Vec<EnumThing>) {
//     //     mdb_println!("addr: {}", addr);
//     //     mdb_println!("flags: {}", flags);
//     //     mdb_println!("[");
//     //     for arg in args {
//     //         mdb_println!("  {}", arg);
//     //     }
//     //     mdb_println!("]");
//     // }
// }

#[no_mangle]
pub extern "C" fn _mdb_init() -> *const mdb_modinfo_t {
    let modinfo = Modinfo::default().with_walker::<TokioTaskWalker>();

    modinfo.to_native()
}
// /// this is the description
// #[mdb_magic]
// fn potato(addr: u64, flags: u16, args: Vec<EnumThings>) {}

/// descr
struct Tokio {
    // #[clapish(long, short = "f")]
    foo: String,

    yes: bool,
    // everythingelse: Vec<stuff>,
}

// impl DcmdFn for Tokio {
//     fn dcmd(&self, addr: u64, flags: u16) {}
// }

// impl DcmdComplete for Tokio{
//     fn complete();
// }

// what the macro emits

// extern "C" fn tokio_dcmd_dcmd(
//     addr: uintptr_t,
//     flags: c_uint,
//     argc: c_int,
//     args: *const mdb_arg_t,
// ) -> c_int {
//     // marshall arguments from args -> TokioDcmd
//     dcmd.dcmd(addr, flags);
//     todo!()
// }

// const TOKIO_DCMD: mdb_dcmd_t = mdb_dcmd_t {
//     dc_name: b"tokio\0".as_ptr(),
//     dc_usage: b"from docs".as_ptr(),
//     dc_descr: todo!(),
//     dc_funcp: todo!(),
//     dc_help: todo!(),
//     dc_tabp: todo!(),
// };

// impl Tokio {
//     const fn raw_thing() -> *const mdb_dcmd_t {
//         TOKIO_DCMD.as_ptr()
//     }
// }
