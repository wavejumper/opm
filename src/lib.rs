extern crate pd_external_rs;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};

mod ffi;
use std::thread;

fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}


#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    ffi::post("Starting server on port 9000");
    thread::spawn(move || {
        HttpServer::new(|| {
            App::new()
                .route("/", web::get().to(greet))
                .route("/{name}", web::get().to(greet))
        })
        .bind("0.0.0.0:9000")
        .expect("Can not bind to port 9000")
        .run()
        .unwrap();
    });
}