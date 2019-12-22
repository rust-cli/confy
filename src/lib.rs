//! Zero-boilerplate configuration management
//!
//! ## Why?
//!
//! There are a lot of different requirements when
//! selecting, loading and writing a config,
//! depending on the operating system and other
//! environment factors.
//!
//! In many applications this burden is left to you,
//! the developer of an application, to figure out
//! where to place the configuration files.
//!
//! This is where `confy` comes in.
//!
//! ## Idea
//!
//! `confy` takes care of figuring out operating system
//! specific and environment paths before reading and
//! writing a configuration.
//!
//! It gives you easy access to a configuration file
//! which is mirrored into a Rust `struct` via [serde].
//! This way you only need to worry about the layout of
//! your configuration, not where and how to store it.
//!
//! [serde]: https://docs.rs/crates/serde
//!
//! `confy` uses the [`Default`] trait in Rust to automatically
//! create a new configuration, if none is available to read
//! from yet.
//! This means that you can simply assume your application
//! to have a configuration, which will be created with
//! default values of your choosing, without requiring
//! any special logic to handle creation.
//!
//! [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
//!
//! ```rust
//! #[derive(Serialize, Deserialize)]
//! struct MyConfig {
//!     version: u8,
//!     api_key: String,
//! }
//!
//! /// `MyConfig` implements `Default`
//! impl ::std::default::Default for MyConfig {
//!     fn default() -> Self { Self { version: 0, api_key: "".into() } }
//! }
//!
//! fn main() -> Result<(), ::std::io::Error> {
//!     let cfg = confy::load("my-app-name")?;
//! }
//! ```
//!
//! Updating the configuration is then done via the [`store`] function.
//!
//! [`store`]: fn.store.html
//!

extern crate cargo_metadata;
extern crate directories;
extern crate serde;
extern crate toml;

mod utils;
use utils::*;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind::NotFound, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfyError {
    BadTomlData(toml::de::Error),
    DirectoryCreationFailed(std::io::Error),
    GeneralLoadError(std::io::Error),
    BadConfigDirectoryStr,
    SerializeTomlError(toml::ser::Error),
    WriteConfigurationFileError(std::io::Error),
    ReadConfigurationFileError(std::io::Error),
    OpenConfigurationFileError(std::io::Error),
    CargoMetadataExecError(cargo_metadata::Error),
    CargoMetadataResolveError,
    CargoMetadataRootError,
}

impl fmt::Display for ConfyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfyError::BadTomlData(e) => write!(f, "Bad TOML data: {}", e),
            ConfyError::DirectoryCreationFailed(e) => write!(f, "Failed to create directory: {}", e),
            ConfyError::GeneralLoadError(_) => write!(f, "Failed to load configuration file."),
            ConfyError::BadConfigDirectoryStr => write!(f, "Failed to convert directory name to str."),
            ConfyError::SerializeTomlError(_) => write!(f, "Failed to serialize configuration data into TOML."),
            ConfyError::WriteConfigurationFileError(_) => write!(f, "Failed to write configuration file."),
            ConfyError::ReadConfigurationFileError(_) => write!(f, "Failed to read configuration file."),
            ConfyError::OpenConfigurationFileError(_) => write!(f, "Failed to open configuration file."),
            ConfyError::CargoMetadataExecError(_) => write!(f, "Failed to get cargo metadata."),
            ConfyError::CargoMetadataResolveError => write!(f, "Failed to get crate's dependency graph."),
            ConfyError::CargoMetadataRootError => write!(f, "Failed to get crate's root dependency."),
        }
    }
}

impl Error for ConfyError {}

/// Load an application configuration from disk
///
/// A new configuration file is created with default values if none
/// exists.
///
/// Errors that are returned from this function are I/O related,
/// for example if the writing of the new configuration fails
/// or `confy` encounters an operating system or environment
/// that it does not support.
///
/// **Note:** The type of configuration needs to be declared in some way
/// that is inferrable by the compiler. Also note that your
/// configuration needs to implement `Default`.
///
/// ```rust,no_run
/// # struct MyConfig {}
/// # impl ::std::default::Default for MyConf {
/// #     fn default() -> Self { Self {} }
/// # }
/// let cfg: MyConfig = confy::load("my-app-name")?;
/// ```
pub fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> Result<T, ConfyError> {
    let root_name = get_root_name()?;

    let project = ProjectDirs::from("rs", &root_name, name);

    let config_dir_str = get_configuration_directory_str(&project)?;

    let path: PathBuf = [config_dir_str, &format!("{}.toml", name)].iter().collect();

    match File::open(&path) {
        Ok(mut cfg) => {
            let cfg_string = cfg
                .get_string()
                .map_err(ConfyError::ReadConfigurationFileError)?;
            let cfg_data = toml::from_str(&cfg_string);
            cfg_data.map_err(ConfyError::BadTomlData)
        }
        Err(ref e) if e.kind() == NotFound => {
            fs::create_dir_all(project.config_dir())
                .map_err(ConfyError::DirectoryCreationFailed)?;
            store(name, T::default())?;
            Ok(T::default())
        }
        Err(e) => Err(ConfyError::GeneralLoadError(e)),
    }
}

/// Save changes made to a configuration object
///
/// This function will update a configuration,
/// with the provided values, and create a new one,
/// if none exists.
///
/// You can also use this function to create a new configuration
/// with different initial values than which are provided
/// by your `Default` trait implementation, or if your
/// configuration structure _can't_ implement `Default`.
///
/// ```rust,no_run
/// # struct MyConf {}
/// let my_cfg = MyConf {};
/// confy::store("my-app-name", my_cfg)?;
/// ```
///
/// Errors returned are I/O errors related to not being
/// able to write the configuration file or if `confy`
/// encounters an operating system or environment it does
/// not support.
pub fn store<T: Serialize>(name: &str, cfg: T) -> Result<(), ConfyError> {
    let root_name = get_root_name()?;

    let project = ProjectDirs::from("rs", &root_name, name);
    fs::create_dir_all(project.config_dir()).map_err(ConfyError::DirectoryCreationFailed)?;

    let config_dir_str = get_configuration_directory_str(&project)?;

    let path: PathBuf = [config_dir_str, &format!("{}.toml", name)].iter().collect();

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(ConfyError::OpenConfigurationFileError)?;
    let s = toml::to_string_pretty(&cfg).map_err(ConfyError::SerializeTomlError)?;
    f.write_all(s.as_bytes())
        .map_err(ConfyError::WriteConfigurationFileError)?;
    Ok(())
}

fn get_configuration_directory_str(project: &ProjectDirs) -> Result<&str, ConfyError> {
    let config_dir_option = project.config_dir().to_str();

    match config_dir_option {
        Some(x) => Ok(x),
        None => Err(ConfyError::BadConfigDirectoryStr),
    }
}

fn get_root_name() -> Result<String, ConfyError> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    let dep_graph = cmd.exec().map_err(ConfyError::CargoMetadataExecError)?;
    
    let package = match dep_graph.resolve {
        Some(p) => Ok(p), 
        None => Err(ConfyError::CargoMetadataResolveError),
    }?;
    
    let package_root = match package.root {
        Some(r) => Ok(r),
        None => Err(ConfyError::CargoMetadataRootError),
    }?;
    //Package root will look like:
    //PackageId { repr: "conf_test 0.1.0 (path+file:///Users/code/conf_test)" }

    let root_name_string = package_root.repr.split(' ').collect::<Vec<&str>>();

    Ok(root_name_string[0].to_string())
}
