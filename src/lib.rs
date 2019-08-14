mod ffi;
mod types;
mod logging;

use std::thread;
use actix_web::{http::header, middleware, App, HttpServer};
use actix_cors::Cors;
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
                .wrap(
                    Cors::new()
                        .allowed_origin("http://0.0.0.0:9000")
                        .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                        .allowed_header(header::CONTENT_TYPE)
                        .max_age(3600)
                )
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