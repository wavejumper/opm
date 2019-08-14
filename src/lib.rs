mod ffi;
mod types;
mod logging;

use std::thread;
use actix_web::{http::header, middleware, App, HttpServer};
use types::{Context};
use logging::{PdLogger};
use log::{info, LevelFilter};

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    static LOGGER: PdLogger = PdLogger;

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Log configuration failed...");

    thread::spawn(move || {
        HttpServer::new(move || {
            let state = Context::new("/home/tscrowley/samples");
            App::new()
                .data(state)
                .wrap(middleware::Logger::default())
        })
        .bind("0.0.0.0:9000")
        .expect("Can not bind to port 9000")
        .run()
        .unwrap();

        info!("Started server on port 9000 :)");
    });
}