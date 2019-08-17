use crate::errors::{IronConfigurationError, ResourceNotFound};
use crate::manifest::{Db, Kit, Kits, Manifest, Sample};
use iron::error::IronError;
use iron::prelude::*;
use iron::mime;
use router::Router;
use std::io::prelude::*;
use std::fs::File;

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

pub struct WavFile {
    bytes: Vec<u8>
}

impl WavFile {
    pub fn new(bytes: Vec<u8>) -> WavFile {
        WavFile { bytes }
    }
}

impl HTTPResponder for IronResult<WavFile> {
    fn unwrap_response(self) -> IronResult<Response> {
        match self {
            Ok(wav) => {
                let content_type = "audio/x-wav".parse::<mime::Mime>().unwrap();
                Ok(Response::with((content_type, iron::status::Ok, wav.bytes)))
            }
            Err(e) => Err(e),
        }
    }
}

impl<T: serde::Serialize> HTTPResponder for IronResult<T> {
    fn unwrap_response(self) -> IronResult<Response> {
        match self {
            Ok(body) => {
                match serde_json::to_string(&body) {
                    Ok(s) => {
                        // todo: figure out the mime! macro, use over this...
                        let content_type = "application/json".parse::<mime::Mime>().unwrap();
                        Ok(Response::with((content_type, iron::status::Ok, s)))
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
    fn play_sample(&self, kit_id: &String, sample_id: &String) -> IronResult<WavFile>;
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

    fn play_sample(&self, kit_id: &String, sample_id: &String) -> IronResult<WavFile> {
        let dir = self.relative_dir;
        let path_str = format!("{}/{}/{}", dir, kit_id, sample_id);
        match File::open(path_str) {
            Ok(mut f) => {
                let mut buffer = Vec::new();
                match f.read_to_end(&mut buffer) {
                    Ok(_) => {
                        let wav_file = WavFile::new(buffer);
                        Ok(wav_file)
                    },
                    Err(_) => {
                        let err = IronError::new(ResourceNotFound, iron::status::InternalServerError);
                        Err(err)
                    }
                }
            },
            Err(_) => {
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

    router.get(
        "/kits/:kit-id/samples/:sample-id/play",
        move |req: &mut Request| {
            let kit_id = extract_query(req, "kit-id")?;
            let sample_id = extract_query(req, "sample-id")?;
            let sample_id = format!("{}.wav", sample_id);
            db.play_sample(&kit_id, &sample_id).unwrap_response()
        },
        "play-sample",
    );

    router
}
