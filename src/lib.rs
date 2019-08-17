extern crate iron;
extern crate router;
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
use log::{info, LevelFilter};
use iron::prelude::*;
use router::Router;
use http::{HTTPController, HTTPResponder};

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    static LOGGER: PdLogger = PdLogger;

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Log configuration failed...");

    thread::spawn(move || {
        info!("Starting server on port 9000");
        let db = Db::new("/home/tscrowley/samples").unwrap();

        let mut router = Router::new();
        router.get("/kits", move |_: &mut Request| db.get_kits().unwrap_response(), "get_kits");
        router.post("/kits", move |_: &mut Request| db.get_kits().unwrap_response(), "post_kits");

        router.get("/kits/:kit-id", move | req: &mut Request |  {
            let kit_id = http::extract_query(req,"kit-id")?;
            db.get_kit(&kit_id).unwrap_response()
        }, "get_kit");

        let mut chain = Chain::new(router);
        chain.link_before(db);
        chain.link_after(db);

        Iron::new(chain).http("localhost:9000").unwrap();
    });
}