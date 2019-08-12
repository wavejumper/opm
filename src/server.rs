#[macro_use]
extern crate juniper;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}