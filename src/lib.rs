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

extern crate directories;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate failure;

mod utils;
use utils::*;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind::NotFound, Write};
use std::path::PathBuf;

/*

impl std::fmt::Display for ConfyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConfyError {}
*/

#[derive(Debug, Fail)]
pub enum ConfyError {
    #[fail(display = "Bad TOML data: {}", _0)]
    BadTomlData(toml::de::Error),

    #[fail(display = "Failed to create directory: {}", _0)]
    DirectoryCreationFailed(std::io::Error),

    #[fail(display = "Failed to load configuration file.")]
    GeneralLoadError(std::io::Error),

    #[fail(display = "Failed to convert directory name to str.")]
    BadConfigDirectoryStr,

    #[fail(display = "Failed to serialize configuration data into TOML.")]
    SerializeTomlError(toml::ser::Error),

    #[fail(display = "Failed to write configuration file.")]
    WriteConfigurationFileError(std::io::Error),

    #[fail(display = "Failed to read configuration file.")]
    ReadConfigurationFileError(std::io::Error),

    #[fail(display = "Failed to open configuration file.")]
    OpenConfigurationFileError(std::io::Error),
}

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
/// let cfg: MyConfig = confy::load("my-app")?;
/// ```
pub fn load<T: Serialize + DeserializeOwned + Default>(name: &str) -> Result<T, ConfyError> {
    let project = ProjectDirs::from("rs", name, name);

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
/// confy::store(my_cfg)?;
/// ```
///
/// Errors returned are I/O errors related to not being
/// able to write the configuration file or if `confy`
/// encounters an operating system or environment it does
/// not support.
pub fn store<T: Serialize>(name: &str, cfg: T) -> Result<(), ConfyError> {
    let project = ProjectDirs::from("rs", name, name);
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
