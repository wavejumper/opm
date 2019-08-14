use log::{Record, Level, Metadata};
use crate::ffi;
use std::ffi::CString;

pub struct PdLogger;

pub fn post(s: &str) {
    unsafe {
        let c_str = CString::new(s).unwrap();
        ffi::post(c_str.as_ptr());
    }
}

impl log::Log for PdLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let s = format!("{} - {}", record.level(), record.args());
            post(s.as_str());
        }
    }

    fn flush(&self) {}
}