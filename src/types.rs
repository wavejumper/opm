use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use log::{info, error};
use std::io::{Error, ErrorKind};
use std::fs;

#[derive(Clone, Serialize, Deserialize)]
pub struct Sample {
    pub name: String
}

#[derive(Serialize, Deserialize,Clone)]
pub struct Kit {
    pub name: String,
    pub dir_name: String,
    pub samples: Vec<Sample>
}

#[derive(Serialize, Deserialize,Clone)]
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
    relative_dir: String
}

impl Db {
    pub fn tx_init(&self) -> std::io::Result<()> {
        let manifest_lock = format!("{}/.manifest.json.lock", self.relative_dir);
        File::create(manifest_lock)?;
        Ok(())
    }

    pub fn aquire_lock(&self) -> std::io::Result<()> {
        let manifest_lock = format!("{}/.manifest.json.lock", self.relative_dir);
        while let Err(_) = File::open(manifest_lock.as_str()) {
            info!("Attempting to aquire lock...");
        }
        info!("Lock aquired...");
        self.tx_init()
    }

    pub fn tx_close(&self) -> std::io::Result<()> {
        let manifest_lock = format!("{}/.manifest.json.lock", self.relative_dir);
        fs::remove_file(manifest_lock)?;
        Ok(())
    }

    pub fn new(relative_dir: &str) -> Result<Self, std::io::Error> {
        let manifest_file = format!("{}/manifest.json", relative_dir);
        File::open(manifest_file)?;
        let db = Db { relative_dir: String::from(relative_dir) };
        Ok(db)
    }

    pub fn read(&self) -> Result<Manifest, std::io::Error> {
        Manifest::open(self.relative_dir.as_str())
    }

    fn commit(&self, manifest: &Manifest) -> std::io::Result<()> {
        info!("Comitting to manifest...");
        let path = format!("{}/manifest.json", self.relative_dir);
        let mut file = File::open(path)?;
        match serde_json::to_string(manifest) {
            Ok(json_str) => {
                file.write_all(json_str.as_bytes())?;
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