extern crate pd_external_rs;

mod ffi;
//extern crate reqwest;

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    ffi::post("Hello world from Rust");
}