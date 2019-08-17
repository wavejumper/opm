extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod errors;
mod ffi;
mod http;
mod logging;
mod manifest;

use iron::prelude::*;
use log::{info, LevelFilter};
use logging::PdLogger;
use manifest::Db;
use std::thread;

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    static LOGGER: PdLogger = PdLogger;

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Log configuration failed...");

    thread::spawn(move || {
        info!("Starting server on port 9000");
        let db = Db::new("/home/tscrowley/samples").unwrap();
        let router = http::app_routes(db);
        let mut chain = Chain::new(router);
        chain.link_before(db);
        chain.link_after(db);
        Iron::new(chain).http("localhost:9000").unwrap();
    });
}
