//! Easy configuration management
//!
//!

extern crate directories;
extern crate serde;
extern crate toml;

mod utils;
use utils::*;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind::NotFound, Write};
use std::path::PathBuf;

/// Load a configuration from the standard OS local scope for
/// the current user.
pub fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> Result<T, IoError> {
    let project = ProjectDirs::from("rs", name, name);

    let path: PathBuf = [
        project.config_dir().to_str().unwrap(),
        &format!("{}.toml", name),
    ].iter()
        .collect();

    match File::open(&path) {
        Ok(mut cfg) => Ok(toml::from_str(&cfg.get_string().unwrap()).unwrap()),
        Err(ref e) if e.kind() == NotFound => {
            fs::create_dir_all(project.config_dir())?;
            store(name, T::default())?;
            Ok(T::default())
        }
        Err(e) => Err(e.into()),
    }
}

/// Store a configuration object
pub fn store<T: Serialize>(name: &str, cfg: T) -> Result<(), IoError> {
    let project = ProjectDirs::from("rs", name, name);
    fs::create_dir_all(project.config_dir())?;

    let path: PathBuf = [
        project.config_dir().to_str().unwrap(),
        &format!("{}.toml", name),
    ].iter()
        .collect();

    let mut f = OpenOptions::new().write(true).create(true).open(path)?;
    let s = toml::to_string_pretty(&cfg).unwrap();
    f.write_all(s.as_bytes())?;
    Ok(())
}
