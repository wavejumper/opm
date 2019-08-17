use crate::types::{Db, Manifest, Kit, Kits};
use iron::prelude::*;
use iron::error::IronError;
use router::Router;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct IronConfigurationError;

impl fmt::Display for IronConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Iron misconfigured!")
    }
}

impl Error for IronConfigurationError {
    fn description(&self) -> &str {
        "Iron Misconfigured"
    }
}

pub fn extract_query(req: &Request, query: &str) -> IronResult<String> {
    match req.extensions.get::<Router>() {
        Some(router) => {
            match router.find(query) {
                Some(q) => {
                    let s = String::from(q);
                    Ok(s)
                },
                None => {
                    let err = IronError::new(IronConfigurationError, iron::status::InternalServerError);
                    Err(err)
                }
            }
        },
        None => {
            let err = IronError::new(IronConfigurationError, iron::status::InternalServerError);
            Err(err)
        }
    }
}

pub trait HTTPResponder {
    fn unwrap_response(self) -> IronResult<Response>;
}

impl<T: serde::Serialize> HTTPResponder for IronResult<T> {
    fn unwrap_response(self) -> IronResult<Response> {
        match self {
            Ok(body) => {
                match serde_json::to_string(&body) {
                    Ok(s) => {
                        // TODO: set headers/etc
                        Ok(Response::with((iron::status::Ok, s)))
                    },
                    Err(e) => {
                        let err = IronError::new(e, iron::status::InternalServerError);
                        Err(err)
                    }

                }
            },
            Err(e) => Err(e)
        }
    }
}

pub trait HTTPController {
    fn get_kits(&self) -> IronResult<Kits>;
    fn get_kit(&self, kit_id: &String) -> IronResult<Kit>;
}

fn read_manifest(db: &Db) -> IronResult<Manifest> {
    match db.read() {
        Ok(manifest) => Ok(manifest),
        Err(e) => {
            let err = IronError::new(e, iron::status::InternalServerError);
            Err(err)
        }
    }
}

impl HTTPController for Db {
    fn get_kits(&self) -> IronResult<Kits> {
        let manifest = read_manifest(self)?;
        Ok(manifest.kits)
    }

    fn get_kit(&self, kit_id: &String) -> IronResult<Kit> {
        let manifest = read_manifest(self)?;
        match manifest.kits.get(kit_id) {
            Some(kit) => Ok(kit.clone()),
            None => {
                let err = IronError::new(IronConfigurationError, iron::status::InternalServerError);
                Err(err)
            }
        }
    }
}