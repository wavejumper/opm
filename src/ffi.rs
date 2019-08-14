// https://rust-embedded.github.io/book/interoperability/c-with-rust.html
// https://github.com/pure-data/externals-howto

use std::os::raw::c_char;

#[link(name="aubio")]
extern "C" {
    pub fn post(fmt: *const c_char);
}