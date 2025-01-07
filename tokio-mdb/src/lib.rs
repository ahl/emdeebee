use std::ffi::{c_int, c_uint, CStr};

use mdb_api::{
    dcmd::{Arg, Code, DcmdFn, Flags},
    sys::{mdb_arg_t, mdb_dcmd_t, mdb_type_t, uintptr_t, DCMD_ERR},
    Addr, Dcmd,
};

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

#[derive(Debug)]
struct Demo {
    // #[dcmd(short, long)]
    foo: String,

    // #[dcmd(short, default = false)]
    verbose: bool,

    // #[dcmd(short, long)]
    maybe_u64: Option<u64>,

    // #[dcmd(short, long, default = 10)]
    defaultable: u64,

    positional: u64,
}

impl DcmdFn for Demo {
    fn call(&self, addr: mdb_api::Addr, flags: mdb_api::dcmd::Flags) -> Code {
        mdb_api::mdb_println!("call(0x{:0x}, 0x{:0x})", addr, flags);
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

    fn from_args(args: Vec<mdb_api::dcmd::Arg>) -> Result<Self, String>
    where
        Self: Sized,
    {
        // Set up the possible arguments.
        //
        // A required argument is created using an `Option`. If that's not
        // specified after processing all arguments, then we bail.
        let mut maybe_foo: Option<String> = None;

        // A flag is a boolean, which defaults to false unless otherwise
        // specified.
        let mut verbose = false;

        // An optional argument _can_ be none, but is otherwise the same.
        let mut maybe_u64: Option<u64> = None;

        // A required argument with a default value.
        let mut defaultable = 10;

        // Store all positionals here as we process the list of arguments.
        let mut positionals = Vec::new();
        mdb_api::mdb_println!("args: {:?}", args);

        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            // Each argument can be one of a few things:
            //
            // - a flag, e.g., `-f`
            // - a value, e.g., the `foo` in `-f foo`.
            // - the literal `--`, indicating all following arguments are
            //   positionals.

            match arg {
                Arg::U64(_) => positionals.push(arg),
                Arg::String(s) if !s.starts_with('-') => positionals.push(arg),
                Arg::String(s) => {
                    // Force all other arguments to be positionals.
                    if s == "--" {
                        break;
                    }

                    // Take either the long- or short-form of the argument.
                    let opt = if let Some(long) = s.strip_prefix("--") {
                        long
                    } else if let Some(short) = s.strip_prefix("-") {
                        if short.len() != 1 {
                            return Err(format!(
                                "flag '{s}' is invalid, all short-form \
                                arguments must be specified alone"
                            ));
                        }
                        short
                    } else {
                        unreachable!();
                    };

                    // Parse as the expected type, into the right variable.
                    match opt {
                        "foo" | "f" => {
                            // Extract the value and convert to the expected
                            // type.
                            let Some(arg) = iter.next() else {
                                return Err(String::from("argument 'foo' requires a value"));
                            };
                            let Ok(val) = String::try_from(arg) else {
                                return Err(format!(
                                    "could not convert value '{arg}' for \
                                    argument 'foo'"
                                ));
                            };
                            if maybe_foo.replace(val).is_some() {
                                return Err(String::from(
                                    "argument 'foo' is specified multiple times",
                                ));
                            }
                        }
                        "verbose" | "v" => verbose = true,
                        "maybe-u64" | "m" => {
                            // Extract the value and convert to the expected
                            // type.
                            let Some(arg) = iter.next() else {
                                return Err(String::from("argument 'maybe-u64' requires a value"));
                            };
                            let Ok(val) = u64::try_from(arg) else {
                                return Err(format!(
                                    "could not convert value '{arg}' for \
                                    argument 'maybe-u64'"
                                ));
                            };
                            if maybe_u64.replace(val).is_some() {
                                return Err(String::from(
                                    "argument 'maybe-u64' is specified multiple times",
                                ));
                            }
                        }
                        "defaultable" | "d" => {
                            // Extract the value and convert to the expected
                            // type.
                            let Some(arg) = iter.next() else {
                                return Err(String::from(
                                    "argument 'defaultable' requires a value",
                                ));
                            };
                            let Ok(val) = u64::try_from(arg) else {
                                return Err(format!(
                                    "could not convert value '{arg}' for \
                                    argument 'defaultable'"
                                ));
                            };
                            defaultable = val;
                        }
                        _ => return Err(format!("unrecognized flag '{s}'")),
                    }
                }
            }
        }

        mdb_api::mdb_println!("Parsing positionals");

        // Process positional arguments, in order.
        let mut positionals = positionals.into_iter().chain(iter);

        let Some(arg) = positionals.next() else {
            return Err(String::from("missing positional argument 'positional'"));
        };
        let positional = arg
            .try_into()
            .map_err(|_| String::from("unexpected type for positional argument 'positional'"))?;

        if positionals.next().is_some() {
            return Err(String::from("too many positional arguments"));
        }

        let self_ = Demo {
            foo: maybe_foo.ok_or_else(|| String::from("missing value for argument 'foo'"))?,
            verbose,
            maybe_u64,
            defaultable,
            positional,
        };
        mdb_api::mdb_println!("{:?}", self_);
        Ok(self_)
    }
}

unsafe extern "C" fn demo_dcmd(
    addr: uintptr_t,
    flags: c_uint,
    argc: c_int,
    argv: *const mdb_arg_t,
) -> c_int {
    // Convert all arguments to our enum, including the actual args and the
    // flags.
    let args = if !argv.is_null() {
        let as_slice = unsafe { std::slice::from_raw_parts(argv, argc as _) };
        let args: Result<Vec<_>, _> = as_slice
            .iter()
            .map(|arg| match arg.a_type {
                mdb_type_t::MDB_TYPE_STRING => {
                    let Ok(s) = CStr::from_ptr(arg.a_un.a_str).to_str().map(String::from) else {
                        mdb_api::mdb_warn!("Failed to convert MDB C-String to Rust");
                        return Err(DCMD_ERR);
                    };
                    Ok(Arg::String(s))
                }
                mdb_type_t::MDB_TYPE_IMMEDIATE => Ok(Arg::U64(arg.a_un.a_val)),
                mdb_type_t::MDB_TYPE_CHAR => Ok(Arg::U64(arg.a_un.a_char as _)),
            })
            .collect();
        let Ok(args) = args else {
            return DCMD_ERR;
        };
        args
    } else {
        Vec::new()
    };

    let flags = Flags::from_bits_truncate(flags);
    mdb_api::mdb_println!(">>{:?}", flags);

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
    demo.call(Addr::new(addr), flags).to_int()

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
