extern crate iron;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod ffi;
mod types;
mod logging;
mod http;

use std::thread;
use types::Db;
use logging::{PdLogger};
use log::LevelFilter;
use iron::prelude::*;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    static LOGGER: PdLogger = PdLogger;

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Log configuration failed...");

    thread::spawn(move || {
        let db = Db::new("/home/tscrowley/samples").unwrap();
        let mut chain = Chain::new(hello_world);
        Iron::new(chain).http("localhost:9000").unwrap();
    });
}