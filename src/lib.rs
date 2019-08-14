extern crate pd_external_rs;
#[macro_use] extern crate juniper;

mod ffi;
mod graphql;
mod types;
mod logging;

use std::sync::Arc;
use std::thread;
use futures::future::Future;
use actix_web::{http::header, middleware, web, App, Error, HttpResponse, HttpServer};
use actix_cors::Cors;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use types::{Db, Manifest};
use crossbeam::crossbeam_channel::bounded;
use crossbeam_utils::atomic::AtomicCell;
use crossbeam::channel::Sender;
use logging::{PdLogger};
use log::{info, LevelFilter};
use crate::graphql::{create_schema, Schema, Context};

fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://0.0.0.0:9000/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[derive(Clone)]
struct State {
    schema: Arc<Schema>,
    context: Context
}

impl State {
    fn new(dir: &str, sender: Sender<Manifest>) -> Result<Self, std::io::Error> {
        let dir_string = String::from(dir);
        let schema = std::sync::Arc::new(create_schema());
        let manifest = Manifest::open(dir)?;
        let manifest_arc = Arc::new(AtomicCell::new(manifest));
        let context = Context { sample_dir: dir_string, manifest: manifest_arc, db_sender: sender };
        Ok(State { schema, context })
    }
}

fn graphql(
    st: web::Data<State>,
    data: web::Json<GraphQLRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let res = data.execute(&st.schema, &st.context);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

#[no_mangle]
pub unsafe extern "C" fn hello_rust() {
    static LOGGER: PdLogger = PdLogger;

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Log configuration failed...");

    info!("Starting server on port 9000 :)");

    let (s, r) = bounded(5);
    let state = State::new("/home/tscrowley/samples", s).unwrap();

    thread::spawn(move || {
        let mut db = Db::new("/home/tscrowley/samples");
        loop {
            match r.recv() {
                Ok(manifest) => {
                    match db.commit(manifest) {
                        Ok(_) => (),
                        Err(_) => {
                            info!("Failed to commit, shutting down stream...");
                            break;
                        }
                    };
                },
                Err(_) => {
                    break;
                }
            }
        }
    });

    thread::spawn(move || {
        HttpServer::new(move || {
            App::new()
                .wrap(
                    Cors::new()
                        .allowed_origin("http://0.0.0.0:9000")
                        .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                        .allowed_header(header::CONTENT_TYPE)
                        .max_age(3600)
                )
                .data(state.clone())
                .wrap(middleware::Logger::default())
                .service(web::resource("/graphql").route(web::post().to_async(graphql)))
                .service(web::resource("/graphiql").route(web::get().to(graphiql)))
        })
        .bind("0.0.0.0:9000")
        .expect("Can not bind to port 9000")
        .run()
        .unwrap();
    });
}