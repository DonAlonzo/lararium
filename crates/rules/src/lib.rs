//use lararium::prelude::*;
//use lararium_abi::prelude::*;

#[no_mangle]
pub extern "C" fn on_load() {
    let Ok(foo) = std::env::var("FOO") else {
        return;
    };
    println!("{foo}");
}

#[no_mangle]
pub extern "C" fn on_publish() {}

#[no_mangle]
pub extern "C" fn on_unload() {}
