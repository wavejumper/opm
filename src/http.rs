use crate::errors::{IronConfigurationError, ResourceNotFound};
use crate::manifest::{Db, Kit, Kits, Manifest, Sample};
use iron::error::IronError;
use iron::prelude::*;
use router::Router;

pub fn extract_query(req: &Request, query: &str) -> IronResult<String> {
    match req.extensions.get::<Router>() {
        Some(router) => match router.find(query) {
            Some(q) => {
                let s = String::from(q);
                Ok(s)
            }
            None => {
                let err = IronError::new(IronConfigurationError, iron::status::InternalServerError);
                Err(err)
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
                    }
                    Err(e) => {
                        let err = IronError::new(e, iron::status::InternalServerError);
                        Err(err)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub trait HTTPController {
    fn get_kits(&self) -> IronResult<Kits>;
    fn get_kit(&self, kit_id: &String) -> IronResult<Kit>;
    fn get_sample(&self, kit_id: &String, sample_id: &String) -> IronResult<Sample>;
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
                let err = IronError::new(ResourceNotFound, iron::status::NotFound);
                Err(err)
            }
        }
    }

    fn get_sample(&self, kit_id: &String, sample_id: &String) -> IronResult<Sample> {
        let kit = self.get_kit(kit_id)?;
        match kit.samples.get(sample_id) {
            Some(sample) => Ok(sample.clone()),
            None => {
                let err = IronError::new(ResourceNotFound, iron::status::NotFound);
                Err(err)
            }
        }
    }
}

pub fn app_routes(db: Db) -> Router {
    let mut router = Router::new();
    router.get(
        "/kits",
        move |_: &mut Request| db.get_kits().unwrap_response(),
        "get_kits",
    );
    router.post(
        "/kits",
        move |_: &mut Request| db.get_kits().unwrap_response(),
        "post_kits",
    );

    router.get(
        "/kits/:kit-id",
        move |req: &mut Request| {
            let kit_id = extract_query(req, "kit-id")?;
            db.get_kit(&kit_id).unwrap_response()
        },
        "get_kit",
    );

    router.get(
        "/kits/:kit-id/samples/:sample-id",
        move |req: &mut Request| {
            let kit_id = extract_query(req, "kit-id")?;
            let sample_id = extract_query(req, "sample-id")?;
            let sample_id = format!("{}.wav", sample_id);
            db.get_sample(&kit_id, &sample_id).unwrap_response()
        },
        "get-sample",
    );

    router
}
