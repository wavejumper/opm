use pd_external_rs;

// https://rust-embedded.github.io/book/interoperability/c-with-rust.html
// https://github.com/pure-data/externals-howto

use std::ffi::CString;

pub fn post(s: &str) {
    let c_str = CString::new(s).unwrap();
    unsafe {
        pd_external_rs::post(c_str.as_ptr());
    }
}