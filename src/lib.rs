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

mod utils;
use utils::*;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind::NotFound, Write};
use std::path::PathBuf;

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
/// struct MyConfig {}
/// impl ::std::default::Default for MyConf {
///     fn default() -> Self { Self {} }
/// }
///
/// let cfg: MyConfig = confy::load("my-app")?;
/// ```
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
/// struct MyConf {}
/// let my_cfg = MyConf { ... };
/// confy::store(my_cfg)?;
/// ```
///
/// Errors returned are I/O errors related to not being
/// able to write the configuration file or if `confy`
/// encounters an operating system or environment it does
/// not support.
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
