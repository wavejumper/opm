use crate::types::{Db, Manifest, Kits};
use iron::prelude::*;

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
}