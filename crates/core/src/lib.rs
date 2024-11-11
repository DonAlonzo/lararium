mod host {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn hello(world: u32);
    }
}

pub fn hello(world: u32) {
    unsafe {
        host::hello(world);
    }
}
