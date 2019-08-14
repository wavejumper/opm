use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use log::{info, error};
use std::io::{Error, ErrorKind};
use crossbeam_utils::atomic::AtomicCell;

#[derive(Serialize, Deserialize, Clone)]
pub struct Sample {
    pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Kit {
    pub name: String,
    pub dir_name: String,
    pub samples: Vec<Sample>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub kits: Vec<Kit>
}

impl Manifest {
    pub fn new() -> Manifest {
        let kits: Vec<Kit> = Vec::new();
        let manifest = Manifest { kits };
        manifest
    }

    pub fn read(file: File) -> std::io::Result<Manifest> {
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        match serde_json::from_str(contents.as_str()) {
            Ok(manifest) => Ok(manifest),
            Err(_) => {
                error!("manifest.json corrupted!");
                let err = Error::new(ErrorKind::InvalidData, "Corrupted manifest.json");
                Err(err)
            }
        }
    }

    pub fn init(relative_dir: &str) -> std::io::Result<Manifest> {
        info!("{}/manifest.json not found, creating...", relative_dir);
        let manifest = Manifest::new();
        let manifest_file_path = format!("{}/manifest.json", relative_dir);
        let mut file = File::create(manifest_file_path)?;
        match serde_json::to_string(&manifest) {
            Ok(json_str) => {
                info!("{}/manifest.json created!", relative_dir);
                file.write_all(json_str.as_bytes())?;
                Ok(manifest)
            },
            Err(_) => {
                error!("Failed to init {}/manifest.json", relative_dir);
                let err = Error::new(ErrorKind::InvalidData, "Failed to init manifest.json");
                Err(err)
            }
        }
    }

    pub fn open(relative_dir: &str) -> std::io::Result<Manifest> {
        info!("Opening {}/manifest.json", relative_dir);
        let manifest = format!("{}/manifest.json", relative_dir);
        match File::open(manifest) {
            Ok(file) => Manifest::read(file),
            Err(_) => Manifest::init(relative_dir)
        }
    }
}

#[derive(Clone)]
pub struct Db {
    relative_dir: String,
    manifest: Manifest
}

impl Db {
    pub fn new(relative_dir: &str) -> Result<Self, std::io::Error> {
        let manifest = Manifest::open(relative_dir)?;
        let db = Db { relative_dir: String::from(relative_dir), manifest: manifest };
        Ok(db)
    }

    pub fn read(self) -> Manifest {
        self.manifest
    }

    pub fn commit(&mut self, manifest: Manifest) -> std::io::Result<()> {
        info!("Comitting to manifest...");
        let path = format!("{}/manifest.json", self.relative_dir);
        let mut file = File::open(path)?;
        match serde_json::to_string(&manifest) {
            Ok(json_str) => {
                file.write_all(json_str.as_bytes())?;
                self.manifest = manifest;
                Ok(())
            },
            Err(_) => {
                error!("Failed to commit to manifest...");
                let err = Error::new(ErrorKind::InvalidData, "Failed to serialize manifest");
                Err(err)
            }
        }
    }
}

pub struct Context {
    pub sample_dir: String,
    pub db: AtomicCell<Db>,
}

impl Context {
    pub fn new(dir: &str) -> Result<Self, std::io::Error> {
        let dir_string = String::from(dir);
        let db = Db::new(dir)?;
        let db_cell = AtomicCell::new(db);
        let context = Context { sample_dir: dir_string, db: db_cell };
        Ok(context)
    }
}