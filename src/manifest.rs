use crate::errors::LockAcquisitionError;
use iron::error::IronError;
use iron::prelude::*;
use iron::{AfterMiddleware, BeforeMiddleware};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::PathBuf;

fn ffi_to_string(s: OsString) -> std::io::Result<String> {
    match s.into_string() {
        Ok(s) => Ok(s),
        Err(_) => {
            let err = std::io::Error::new(std::io::ErrorKind::InvalidData, "Cannot parse OsString");
            Err(err)
        }
    }
}

fn sample_id(s: &String) -> Option<String> {
    match s.find(".wav") {
        Some(idx) => {
            let (id, _) = s.split_at(idx);
            match id.parse::<u32>() {
                Ok(id) => Some(id.to_string()),
                Err(_) => None,
            }
        }
        None => None,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
    pub id: String,
}

impl Sample {
    pub fn new(entry: std::fs::DirEntry) -> Option<Sample> {
        let file_name = ffi_to_string(entry.file_name()).ok()?;
        let sample_id = sample_id(&file_name)?;
        let sample = Sample {
            name: file_name,
            id: sample_id,
        };
        Some(sample)
    }
}

pub type Samples = HashMap<String, Sample>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Kit {
    pub name: String,
    pub dir_name: String,
    pub samples: Samples,
}

fn is_valid_kit_id(s: &String) -> bool {
    match s.find('-') {
        Some(idx) => {
            let (kit, n) = s.split_at(idx + 1);
            match n.parse::<u32>() {
                Ok(id) => kit == "kit-" && id <= 10,
                Err(_) => false,
            }
        }
        None => false,
    }
}

pub type Kits = HashMap<String, Kit>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub kits: Kits,
}
impl Manifest {
    fn new() -> Manifest {
        let kits: Kits = Kits::new();
        let manifest = Manifest { kits };
        manifest
    }

    fn read(file: File) -> std::io::Result<Manifest> {
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        match serde_json::from_str(contents.as_str()) {
            Ok(manifest) => Ok(manifest),
            Err(_) => {
                error!("manifest.json corrupted!");
                let err = std::io::Error::new(ErrorKind::InvalidData, "Corrupted manifest.json");
                Err(err)
            }
        }
    }

    fn resolve_samples(path: PathBuf) -> std::io::Result<Samples> {
        let entries = fs::read_dir(path)?;
        let samples = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let metadata = entry.metadata().ok()?;
                if metadata.is_file() {
                    Sample::new(entry)
                } else {
                    None
                }
            })
            .map(|sample| {
                let s = sample.clone();
                (sample.id, s)
            })
            .collect();

        Ok(samples)
    }

    fn resolve_kits(&mut self, relative_dir: &str) -> std::io::Result<()> {
        // TODO: make more functional...
        for entry in fs::read_dir(relative_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let dir_name = ffi_to_string(entry.path().into_os_string())?;
            let name = ffi_to_string(entry.file_name().to_os_string())?;
            let k = name.clone();
            if is_valid_kit_id(&k) && !metadata.is_file() {
                let samples = Manifest::resolve_samples(entry.path())?;
                let kit = Kit {
                    dir_name,
                    samples,
                    name,
                };
                self.kits.insert(k, kit);
            }
        }
        Ok(())
    }

    fn init(relative_dir: &str) -> std::io::Result<Manifest> {
        info!("{}/manifest.json not found, creating...", relative_dir);
        let mut manifest = Manifest::new();
        manifest.resolve_kits(relative_dir)?;

        let manifest_file_path = format!("{}/manifest.json", relative_dir);
        let mut file = File::create(manifest_file_path)?;
        match serde_json::to_string(&manifest) {
            Ok(json_str) => {
                info!("{}/manifest.json created!", relative_dir);
                file.write_all(json_str.as_bytes())?;
                Ok(manifest)
            }
            Err(_) => {
                error!("Failed to init {}/manifest.json", relative_dir);
                let err =
                    std::io::Error::new(ErrorKind::InvalidData, "Failed to init manifest.json");
                Err(err)
            }
        }
    }

    pub fn open(relative_dir: &str) -> std::io::Result<Manifest> {
        debug!("Opening {}/manifest.json", relative_dir);
        let manifest = format!("{}/manifest.json", relative_dir);
        match File::open(manifest) {
            Ok(file) => Manifest::read(file),
            Err(_) => Manifest::init(relative_dir),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Db {
    pub relative_dir: &'static str,
}

impl Db {
    fn lock_file(&self) -> String {
        format!("{}/mainfest.json.lock", self.relative_dir)
    }

    pub fn aquire_lock(&self) -> std::io::Result<()> {
        debug!("Aquring lock");
        if fs::metadata(self.lock_file().as_str()).is_ok() {
            let err = std::io::Error::new(std::io::ErrorKind::AlreadyExists, "lock already exists");
            Err(err)
        } else {
            File::create(self.lock_file())?;
            Ok(())
        }
    }

    pub fn release_lock(&self) -> std::io::Result<()> {
        debug!("Releasing lock");
        fs::remove_file(self.lock_file())?;
        Ok(())
    }

    pub fn new(relative_dir: &'static str) -> Result<Self, std::io::Error> {
        Manifest::open(relative_dir)?;
        let db = Db {
            relative_dir: relative_dir,
        };
        Ok(db)
    }

    pub fn read(&self) -> Result<Manifest, std::io::Error> {
        Manifest::open(self.relative_dir)
    }

    fn commit(&self, manifest: &Manifest) -> std::io::Result<()> {
        info!("Comitting to manifest...");
        let path = format!("{}/manifest.json", self.relative_dir);
        let mut file = File::open(path)?;
        match serde_json::to_string(manifest) {
            Ok(json_str) => {
                file.write_all(json_str.as_bytes())?;
                Ok(())
            }
            Err(_) => {
                error!("Failed to commit to manifest...");
                let err =
                    std::io::Error::new(ErrorKind::InvalidData, "Failed to serialize manifest");
                Err(err)
            }
        }
    }
}

impl BeforeMiddleware for Db {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        match req.method {
            iron::method::Post => match self.aquire_lock() {
                Ok(_) => Ok(()),
                Err(_) => {
                    let err =
                        IronError::new(LockAcquisitionError, iron::status::InternalServerError);
                    Err(err)
                }
            },
            _ => Ok(()),
        }
    }
}

impl AfterMiddleware for Db {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match req.method {
            iron::method::Post => match self.release_lock() {
                Ok(_) => Ok(res),
                Err(_) => {
                    warn!("Failed to release lock");
                    let err =
                        IronError::new(LockAcquisitionError, iron::status::InternalServerError);
                    Err(err)
                }
            },
            _ => Ok(res),
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        if req.method == iron::method::Post && !err.error.is::<LockAcquisitionError>() {
            match self.release_lock() {
                _ => (),
            };
        };

        Err(err)
    }
}
