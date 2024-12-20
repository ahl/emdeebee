use std::ffi::{c_int, c_uint, CStr};

use mdb_api::{dcmd::{Arg, Code, DcmdFn, Flags}, sys::{mdb_arg_t, mdb_dcmd_t, mdb_type_t, uintptr_t, DCMD_ERR}, Addr, Dcmd};

mod walk_tasks;

// const MODINFO: MdbModInfo = MdbModInfo {
//     dcmds: &[MdbDcmd {
//         name: "test1",
//         usage: "test1 usage",
//         description: "this is test1!",
//         command: ??
//     }],
// };

//struct HelloDcmd;

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

mdb_api::dmod! {
    commands = [Demo],
    walkers = [walk_tasks::TokioTaskWalker],
}

// /// this is the description
// #[mdb_magic]
// fn potato(addr: u64, flags: u16, args: Vec<EnumThings>) {}


struct Demo {
    // #[dcmd(short, long)]
    foo: String,

    // #[dcmd(short)]
    verbose: bool,

    // #[dcmd(short, long)]
    maybe_u64: Option<u64>,

    // #[dcmd(short, long, default = 10)]
    defaultable: u64,

    positional: u64,
}

impl DcmdFn for Demo {
    fn call(&self, addr: mdb_api::Addr, _flags: mdb_api::dcmd::Flags) -> Code {
        mdb_api::mdb_println!("0x{:0x}", addr);
        Code::Ok
    }
}

impl Dcmd for Demo {
    fn linkage() -> mdb_dcmd_t {
        mdb_dcmd_t {
            dc_name: b"demo\0".as_ptr().cast(),
            dc_usage: b"[-f | --foo FOO ] [-v | --verbose]\0".as_ptr().cast(),
            dc_descr: b"do a demo\0".as_ptr().cast(),
            dc_funcp: Some(demo_dcmd),
            dc_help: None,
            dc_tabp: None,
        }
    }

    fn from_args(args: Vec<mdb_api::dcmd::Arg>) -> Result<Self, String> where Self: Sized {
        let mut maybe_foo: Option<String> = None;
        let mut verbose = false;
        let mut maybe_u64: Option<u64> = None;
        let mut defaultable = 10;
        let positional;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg {
                Arg::String(s) => {
                    // Start positionals!
                    if s == "--" {
                        break;
                    }

                    // Is this a long flag?
                    if let Some(long_name) = s.strip_prefix("--") {
                        match long_name {
                            "foo" => {
                                // get next and replace
                                let Some(next) = iter.next() else {
                                    return Err(String::from("argument 'foo' is missing a value"));
                                };
                                let Ok(as_val) = String::try_from(next) else {
                                    return Err(String::from("unexpected type for argument 'foo'"));
                                };
                                let old = maybe_foo.replace(as_val);
                                if old.is_some() {
                                    return Err(String::from("argument 'foo' is specified more than once"));
                                }
                                continue;
                            }
                            "verbose" => {
                                verbose = true;
                                continue;
                            }
                            _ => return Err(format!("unrecognized argument: '{}'", long_name)),
                        }
                    }

                    // Is this a short flag
                    if s.len() == 2 {
                        if let Some(flag) = s.strip_prefix("-") {
                            // match and do the same as above.
                            match flag {
                                "f" => {
                                    // get next and replace
                                    let Some(next) = iter.next() else {
                                        return Err(String::from("argument 'foo' is missing a value"));
                                    };
                                    let Ok(as_val) = String::try_from(next) else {
                                        return Err(String::from("unexpected type for argument 'foo'"));
                                    };
                                    let old = maybe_foo.replace(as_val);
                                    if old.is_some() {
                                        return Err(String::from("argument 'foo' is specified more than once"));
                                    }
                                    continue;
                                }
                                "v" => {
                                    verbose = true;
                                    continue;
                                }
                                _ => return Err(format!("unrecognized flag: '{}'", flag)),
                            }
                        }
                    }
                }
                Arg::U64(_) => todo!(),
            }
        }

        // Process positional arguments, in order.
        let Some(arg) = iter.next() else {
            return Err(String::from("missing positional argument 'positional_u64'"));
        };
        positional = arg
            .try_into()
            .map_err(|_| String::from("unexpected type for positional argument 'positional'"))?;

        Ok(Demo {
            foo: maybe_foo.ok_or_else(|| {
                String::from("missing value for argument 'foo'")
            })?,
            verbose,
            maybe_u64,
            defaultable,
            positional,
        })
    }
}

unsafe extern "C" fn demo_dcmd(
    addr: uintptr_t,
    flags: c_uint,
    argc: c_int,
    argv: *const mdb_arg_t,
) -> c_int
{
    // Convert all arguments to our enum, including the actual args and the
    // flags.
    let args = if !argv.is_null() {
        let as_slice = unsafe { std::slice::from_raw_parts(argv, argc as _) };
        let args: Result<Vec<_>, _> = as_slice
            .iter()
            .map(|arg| {
                match arg.a_type {
                    mdb_type_t::MDB_TYPE_STRING => {
                        let Ok(s) = CStr::from_ptr(arg.a_un.a_str)
                            .to_str()
                            .map(String::from)
                        else {
                            mdb_api::mdb_warn!("Failed to convert MDB C-String to Rust");
                            return Err(DCMD_ERR);
                        };
                        Ok(Arg::String(s))
                    }
                    mdb_type_t::MDB_TYPE_IMMEDIATE => {
                        Ok(Arg::U64(arg.a_un.a_val))
                    }
                    mdb_type_t::MDB_TYPE_CHAR => {
                        Ok(Arg::U64(arg.a_un.a_char as _))
                    }
                }
            })
            .collect();
        let Ok(args) = args else {
            return DCMD_ERR;
        };
        args
    } else {
        Vec::new()
    };

    // Pass these to the derived arg conversion method.
    let demo = match Demo::from_args(args) {
        Ok(demo) => demo,
        Err(e) => {
            mdb_api::mdb_warn!(
                "Failed to construct type `Demo` \
                from mdb args: {}",
                e,
            );
            return DCMD_ERR;
        }
    };


    // Call the fucker
    demo.call(Addr::new(addr), Flags::from_bits_truncate(flags)).to_int()




    // Parse the arguments from argv, into `Demo`.
    //  - build the 4-tuple of args to mdb_getopt for each arg
    //    - option letter
    //    - option type
    //    - other two based on that
    //
    //  - call mdb_getopts, return on failure
    //  - for each argument:
    //    - try to convert the destination into the argument
    //    - warn if that fails and return
    //    - assign to a local with the same name as the field
    //
    //  - assign all locals to build the struct
    // Call `demo.command(addr, flags)`
    // return mapped result
}


/*
/// descr
struct Tokio {
    // #[clapish(long, short = "f")]
    foo: String,

    yes: bool,
    // everythingelse: Vec<stuff>,
}
*/

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
