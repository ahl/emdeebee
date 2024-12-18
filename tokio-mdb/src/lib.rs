use mdb_api::{mdb_modinfo_t, MdbDcmd, MdbDcmdImpl, MdbModInfo};

const MODINFO: MdbModInfo = MdbModInfo {
    dcmds: &[MdbDcmd {
        name: "test1",
        usage: "test1 usage",
        description: "this is test1!",
        command: ??
    }],
};

struct HelloDcmd;

impl MdbDcmdImpl for HelloDcmd{
    fn name() -> String {
        todo!()
    }

    fn usage() -> String {
        todo!()
    }

    fn description() -> String {
        todo!()
    }

    fn command() {
        todo!()
    }
}

#[no_mangle]
pub extern "C" fn _mdb_init() -> *const mdb_modinfo_t {

    MODINFO.to_native()
}
