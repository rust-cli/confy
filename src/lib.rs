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
//! [serde]: https://docs.rs/serde
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
//! ```rust,no_run
//! use serde_derive::{Serialize, Deserialize};
//!
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
//! fn main() -> Result<(), confy::ConfyError> {
//!     let cfg: MyConfig = confy::load("my-app-name", None)?;
//!     Ok(())
//! }
//! ```
//!
//! Serde is a required dependency, and can be added with either the `serde_derive` crate or `serde` crate with feature derive as shown below
//!```toml,no_run
//![dependencies]
//!serde = { version = "1.0.152", features = ["derive"] } # <- Only one serde version needed (serde or serde_derive)
//!serde_derive = "1.0.152" # <- Only one serde version needed (serde or serde_derive)
//!confy = "^0.6"
//!```
//! Updating the configuration is then done via the [`store`] function.
//!
//! [`store`]: fn.store.html
//!
//! ## Features
//!
//! Exactly **one** of the features has to be enabled from the following table.
//!
//! ### Tip
//! to add this crate to your project with the default, toml config do the following: `cargo add confy`, otherwise do something like: `cargo add confy --no-default-features --features yaml_conf`, for more info, see [cargo docs on features]
//! 
//! [cargo docs on features]: https://docs.rust-lang.org/cargo/reference/resolver.html#features
//! 
//! feature | file format | description
//! ------- | ----------- | -----------
//! **default**: `toml_conf` | [toml] | considered a reasonable default, uses the standard-compliant [`toml` crate]
//! `yaml_conf` | [yaml] | uses the [`serde_yaml` crate]
//! `ron_conf` | [ron] | Rusty Object Notation, uses the [`ron` crate]
//! `basic_toml_conf` | [toml] | alternative to the default `toml_conf`, instead of using the [`toml` crate], the [`basic_toml` crate] is used, in order to cut down on the number of dependencies, speed up compilation and shrink binary size. **_DISCLAIMER_**: this crate is **not** standard compliant, **nor** maintained, otherwise should work fine in most situations.
//!
//! [toml]: https://toml.io
//! [`toml` crate]: https://docs.rs/toml
//! [yaml]: https://yaml.org
//! [`serde_yaml` crate]: https://docs.rs/serde_yaml
//! [ron]: https://docs.rs/ron
//! [`ron` crate]: https://docs.rs/ron
//! [`basic_toml` crate]: https://docs.rs/basic_toml

mod utils;
use utils::*;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{ErrorKind::NotFound, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[cfg(feature = "toml_conf")]
use toml::{
    de::Error as TomlDeErr, from_str as toml_from_str, ser::Error as TomlSerErr,
    to_string_pretty as toml_to_string_pretty,
};

#[cfg(feature = "basic_toml_conf")]
use basic_toml::{
    from_str as toml_from_str, to_string as toml_to_string_pretty, Error as TomlDeErr,
    Error as TomlSerErr,
};

#[cfg(not(any(
    feature = "toml_conf",
    feature = "basic_toml_conf",
    feature = "yaml_conf",
    feature = "ron_conf"
)))]
compile_error!(
    "Exactly one config language feature must be enabled to use \
confy. Please enable one of either the `toml_conf`, `yaml_conf`, \
, `ron_conf` or `toml_basic_conf` features."
);

#[cfg(any(
    all(feature = "toml_conf", feature = "basic_toml_conf"),
    all(
        any(feature = "toml_conf", feature = "basic_toml_conf"),
        feature = "yaml_conf"
    ),
    all(
        any(feature = "toml_conf", feature = "basic_toml_conf"),
        feature = "ron_conf"
    ),
    all(feature = "ron_conf", feature = "yaml_conf"),
))]
compile_error!(
    "Exactly one config language feature must be enabled to compile \
confy.  Please disable one of either the `toml_conf`, `basic_toml_conf`, `yaml_conf`, or `ron_conf` features. \
NOTE: `toml_conf` is a default feature, so disabling it might mean switching off \
default features for confy in your Cargo.toml"
);

#[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
const EXTENSION: &str = "toml";

#[cfg(feature = "yaml_conf")]
const EXTENSION: &str = "yml";

#[cfg(feature = "ron_conf")]
const EXTENSION: &str = "ron";

/// The errors the confy crate can encounter.
#[derive(Debug, Error)]
pub enum ConfyError {
    #[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
    #[error("Bad TOML data")]
    BadTomlData(#[source] TomlDeErr),

    #[cfg(feature = "yaml_conf")]
    #[error("Bad YAML data")]
    BadYamlData(#[source] serde_yaml::Error),

    #[cfg(feature = "ron_conf")]
    #[error("Bad RON data")]
    BadRonData(#[source] ron::error::SpannedError),

    #[error("Failed to create directory")]
    DirectoryCreationFailed(#[source] std::io::Error),

    #[error("Failed to load configuration file")]
    GeneralLoadError(#[source] std::io::Error),

    #[error("Bad configuration directory: {0}")]
    BadConfigDirectory(String),

    #[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
    #[error("Failed to serialize configuration data into TOML")]
    SerializeTomlError(#[source] TomlSerErr),

    #[cfg(feature = "yaml_conf")]
    #[error("Failed to serialize configuration data into YAML")]
    SerializeYamlError(#[source] serde_yaml::Error),

    #[cfg(feature = "ron_conf")]
    #[error("Failed to serialize configuration data into RON")]
    SerializeRonError(#[source] ron::error::Error),

    #[error("Failed to write configuration file")]
    WriteConfigurationFileError(#[source] std::io::Error),

    #[error("Failed to read configuration file")]
    ReadConfigurationFileError(#[source] std::io::Error),

    #[error("Failed to open configuration file")]
    OpenConfigurationFileError(#[source] std::io::Error),

    #[error("Failed to set configuration file permissions")]
    SetPermissionsFileError(#[source] std::io::Error),
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
/// that is inferable by the compiler. Also note that your
/// configuration needs to implement `Default`.
///
/// ```rust,no_run
/// # use confy::ConfyError;
/// # use serde_derive::{Serialize, Deserialize};
/// # fn main() -> Result<(), ConfyError> {
/// #[derive(Default, Serialize, Deserialize)]
/// struct MyConfig {}
///
/// let cfg: MyConfig = confy::load("my-app-name", None)?;
/// # Ok(())
/// # }
/// ```
pub fn load<'a, T: Serialize + DeserializeOwned + Default>(
    app_name: &str,
    config_name: impl Into<Option<&'a str>>,
) -> Result<T, ConfyError> {
    get_configuration_file_path(app_name, config_name).and_then(load_path)
}

/// Load an application configuration from a specified path.
///
/// A new configuration file is created with default values if none
/// exists.
///
/// This is an alternate version of [`load`] that allows the specification of
/// an arbitrary path instead of a system one.  For more information on errors
/// and behavior, see [`load`]'s documentation.
///
/// [`load`]: fn.load.html
pub fn load_path<T: Serialize + DeserializeOwned + Default>(
    path: impl AsRef<Path>,
) -> Result<T, ConfyError> {
    match File::open(&path) {
        Ok(mut cfg) => {
            let cfg_string = cfg
                .get_string()
                .map_err(ConfyError::ReadConfigurationFileError)?;

            #[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
            {
                let cfg_data = toml_from_str(&cfg_string);
                cfg_data.map_err(ConfyError::BadTomlData)
            }
            #[cfg(feature = "yaml_conf")]
            {
                let cfg_data = serde_yaml::from_str(&cfg_string);
                cfg_data.map_err(ConfyError::BadYamlData)
            }
            #[cfg(feature = "ron_conf")]
            {
                let cfg_data = ron::from_str(&cfg_string);
                cfg_data.map_err(ConfyError::BadRonData)
            }
        }
        Err(ref e) if e.kind() == NotFound => {
            if let Some(parent) = path.as_ref().parent() {
                fs::create_dir_all(parent).map_err(ConfyError::DirectoryCreationFailed)?;
            }
            let cfg = T::default();
            store_path(path, &cfg)?;
            Ok(cfg)
        }
        Err(e) => Err(ConfyError::GeneralLoadError(e)),
    }
}

/// Load an application configuration from a specified path.
///
/// A new configuration file is created with `op`'s result if none
/// exists or file content is incorrect.
///
/// This is an alternate version of [`load`] that allows the specification of
/// an arbitrary path instead of a system one.  For more information on errors
/// and behavior, see [`load`]'s documentation.
///
/// [`load`]: fn.load.html
pub fn load_or_else<T, F>(path: impl AsRef<Path>, op: F) -> Result<T, ConfyError>
where
    T: DeserializeOwned + Serialize,
    F: FnOnce() -> T,
{
    let path_ref = path.as_ref();
    let load_value = || {
        let cfg = op();
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(ConfyError::DirectoryCreationFailed)?;
        }
        store_path(path_ref, &cfg)?;
        Ok(cfg)
    };

    match File::open(path_ref) {
        Ok(mut cfg) => {
            let mut load_from_file = || {
                let cfg_string = cfg
                    .get_string()
                    .map_err(ConfyError::ReadConfigurationFileError)?;

                #[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
                {
                    let cfg_data = toml_from_str(&cfg_string);
                    cfg_data.map_err(ConfyError::BadTomlData)
                }
                #[cfg(feature = "yaml_conf")]
                {
                    let cfg_data = serde_yaml::from_str(&cfg_string);
                    cfg_data.map_err(ConfyError::BadYamlData)
                }
                #[cfg(feature = "ron_conf")]
                {
                    let cfg_data = ron::from_str(&cfg_string);
                    cfg_data.map_err(ConfyError::BadRonData)
                }
            };
            load_from_file().or_else(|_| load_value())
        }
        Err(ref e) if e.kind() == NotFound => load_value(),
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
/// # use serde_derive::{Serialize, Deserialize};
/// # use confy::ConfyError;
/// # fn main() -> Result<(), ConfyError> {
/// #[derive(Serialize, Deserialize)]
/// struct MyConf {}
///
/// let my_cfg = MyConf {};
/// confy::store("my-app-name", None, my_cfg)?;
/// # Ok(())
/// # }
/// ```
///
/// Errors returned are I/O errors related to not being
/// able to write the configuration file or if `confy`
/// encounters an operating system or environment it does
/// not support.
pub fn store<'a, T: Serialize>(
    app_name: &str,
    config_name: impl Into<Option<&'a str>>,
    cfg: T,
) -> Result<(), ConfyError> {
    let path = get_configuration_file_path(app_name, config_name)?;
    store_path(path, cfg)
}

/// Save changes made to a configuration object at a specified path
///
/// This is an alternate version of [`store`] that allows the specification of
/// file permissions that must be set. For more information on errors and
/// behavior, see [`store`]'s documentation.
///
/// [`store`]: fn.store.html
pub fn store_perms<'a, T: Serialize>(
    app_name: &str,
    config_name: impl Into<Option<&'a str>>,
    cfg: T,
    perms: Permissions,
) -> Result<(), ConfyError> {
    let path = get_configuration_file_path(app_name, config_name)?;
    store_path_perms(path, cfg, perms)
}

/// Save changes made to a configuration object at a specified path
///
/// This is an alternate version of [`store`] that allows the specification of
/// an arbitrary path instead of a system one.  For more information on errors
/// and behavior, see [`store`]'s documentation.
///
/// [`store`]: fn.store.html
pub fn store_path<T: Serialize>(path: impl AsRef<Path>, cfg: T) -> Result<(), ConfyError> {
    do_store(path.as_ref(), cfg, None)
}

/// Save changes made to a configuration object at a specified path
///
/// This is an alternate version of [`store_path`] that allows the
/// specification of file permissions that must be set. For more information on
/// errors and behavior, see [`store`]'s documentation.
///
/// [`store_path`]: fn.store_path.html
pub fn store_path_perms<T: Serialize>(
    path: impl AsRef<Path>,
    cfg: T,
    perms: Permissions,
) -> Result<(), ConfyError> {
    do_store(path.as_ref(), cfg, Some(perms))
}

fn do_store<T: Serialize>(
    path: &Path,
    cfg: T,
    perms: Option<Permissions>,
) -> Result<(), ConfyError> {
    let config_dir = path
        .parent()
        .ok_or_else(|| ConfyError::BadConfigDirectory(format!("{path:?} is a root or prefix")))?;
    fs::create_dir_all(config_dir).map_err(ConfyError::DirectoryCreationFailed)?;

    let s;
    #[cfg(any(feature = "toml_conf", feature = "basic_toml_conf"))]
    {
        s = toml_to_string_pretty(&cfg).map_err(ConfyError::SerializeTomlError)?;
    }
    #[cfg(feature = "yaml_conf")]
    {
        s = serde_yaml::to_string(&cfg).map_err(ConfyError::SerializeYamlError)?;
    }
    #[cfg(feature = "ron_conf")]
    {
        let pretty_cfg = ron::ser::PrettyConfig::default();
        s = ron::ser::to_string_pretty(&cfg, pretty_cfg).map_err(ConfyError::SerializeRonError)?;
    }

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(ConfyError::OpenConfigurationFileError)?;

    if let Some(p) = perms {
        f.set_permissions(p)
            .map_err(ConfyError::SetPermissionsFileError)?;
    }

    f.write_all(s.as_bytes())
        .map_err(ConfyError::WriteConfigurationFileError)?;
    Ok(())
}

/// Get the configuration file path used by [`load`] and [`store`]
///
/// This is useful if you want to show where the configuration file is to your user.
///
/// [`load`]: fn.load.html
/// [`store`]: fn.store.html
pub fn get_configuration_file_path<'a>(
    app_name: &str,
    config_name: impl Into<Option<&'a str>>,
) -> Result<PathBuf, ConfyError> {
    let config_name = config_name.into().unwrap_or("default-config");
    let project = ProjectDirs::from("rs", "", app_name).ok_or_else(|| {
        ConfyError::BadConfigDirectory("could not determine home directory path".to_string())
    })?;

    let config_dir_str = get_configuration_directory_str(&project)?;

    let path = [config_dir_str, &format!("{config_name}.{EXTENSION}")]
        .iter()
        .collect();

    Ok(path)
}

fn get_configuration_directory_str(project: &ProjectDirs) -> Result<&str, ConfyError> {
    let path = project.config_dir();
    path.to_str()
        .ok_or_else(|| ConfyError::BadConfigDirectory(format!("{path:?} is not valid Unicode")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serializer;
    use serde_derive::{Deserialize, Serialize};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[derive(PartialEq, Default, Debug, Serialize, Deserialize)]
    struct ExampleConfig {
        name: String,
        count: usize,
    }

    /// Run a test function with a temporary config path as fixture.
    fn with_config_path(test_fn: fn(&Path)) {
        let config_dir = tempfile::tempdir().expect("creating test fixture failed");
        // config_path should roughly correspond to the result of `get_configuration_file_path("example-app", "example-config")`
        let config_path = config_dir
            .path()
            .join("example-app")
            .join("example-config")
            .with_extension(EXTENSION);
        test_fn(&config_path);
        config_dir.close().expect("removing test fixture failed");
    }

    /// [`load_path`] loads [`ExampleConfig`].
    #[test]
    fn load_path_works() {
        with_config_path(|path| {
            let config: ExampleConfig = load_path(path).expect("load_path failed");
            assert_eq!(config, ExampleConfig::default());
        })
    }

    /// [`load_or_else`] loads [`ExampleConfig`].
    #[test]
    fn load_or_else_works() {
        with_config_path(|path| {
            let the_value = || ExampleConfig {
                name: "a".to_string(),
                count: 5,
            };

            let config: ExampleConfig = load_or_else(path, the_value).expect("load_or_else failed");
            assert_eq!(config, the_value());
        });

        with_config_path(|path| {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut file = File::create(path).expect("creating file failed");
            file.write("some normal text".as_bytes())
                .expect("write to file failed");
            drop(file);

            let the_value = || ExampleConfig {
                name: "a".to_string(),
                count: 5,
            };

            let config: ExampleConfig = load_or_else(path, the_value).expect("load_or_else failed");
            assert_eq!(config, the_value());
        })
    }

    /// [`store_path`] stores [`ExampleConfig`].
    #[test]
    fn test_store_path() {
        with_config_path(|path| {
            let config: ExampleConfig = ExampleConfig {
                name: "Test".to_string(),
                count: 42,
            };
            store_path(path, &config).expect("store_path failed");
            let loaded = load_path(path).expect("load_path failed");
            assert_eq!(config, loaded);
        })
    }

    /// [`store_path_perms`] stores [`ExampleConfig`], with only read permission for owner (UNIX).
    #[test]
    #[cfg(unix)]
    fn test_store_path_perms() {
        with_config_path(|path| {
            let config: ExampleConfig = ExampleConfig {
                name: "Secret".to_string(),
                count: 16549,
            };
            store_path_perms(path, &config, Permissions::from_mode(0o600))
                .expect("store_path_perms failed");
            let loaded = load_path(path).expect("load_path failed");
            assert_eq!(config, loaded);
        })
    }

    /// [`store_path_perms`] stores [`ExampleConfig`], as read-only.
    #[test]
    fn test_store_path_perms_readonly() {
        with_config_path(|path| {
            let config: ExampleConfig = ExampleConfig {
                name: "Soon read-only".to_string(),
                count: 27115,
            };
            store_path(path, &config).expect("store_path failed");

            let metadata = fs::metadata(path).expect("reading metadata failed");
            let mut permissions = metadata.permissions();
            permissions.set_readonly(true);

            store_path_perms(path, &config, permissions).expect("store_path_perms failed");

            assert!(fs::metadata(path)
                .expect("reading metadata failed")
                .permissions()
                .readonly());
        })
    }

    /// [`store_path`] fails when given a root path.
    #[test]
    fn test_store_path_root_error() {
        let err = store_path(PathBuf::from("/"), &ExampleConfig::default())
            .expect_err("store_path should fail");
        assert_eq!(
            err.to_string(),
            r#"Bad configuration directory: "/" is a root or prefix"#,
        )
    }

    struct CannotSerialize;

    impl Serialize for CannotSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            use serde::ser::Error;
            Err(S::Error::custom("cannot serialize CannotSerialize"))
        }
    }

    /// Verify that if you call store_path() with an object that fails to serialize,
    /// the file on disk will not be overwritten or truncated.
    #[test]
    fn test_store_path_atomic() -> Result<(), ConfyError> {
        let tmp = tempfile::NamedTempFile::new().expect("Failed to create NamedTempFile");
        let path = tmp.path();
        let message = "Hello world!";

        // Write to file.
        {
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .map_err(ConfyError::OpenConfigurationFileError)?;

            f.write_all(message.as_bytes())
                .map_err(ConfyError::WriteConfigurationFileError)?;

            f.flush().map_err(ConfyError::WriteConfigurationFileError)?;
        }

        // Call store_path() to overwrite file with an object that fails to serialize.
        let store_result = store_path(path, CannotSerialize);
        assert!(matches!(store_result, Err(_)));

        // Ensure file was not overwritten.
        let buf = {
            let mut f = OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(ConfyError::OpenConfigurationFileError)?;

            let mut buf = String::new();

            use std::io::Read;
            f.read_to_string(&mut buf)
                .map_err(ConfyError::ReadConfigurationFileError)?;
            buf
        };

        assert_eq!(buf, message);
        Ok(())
    }

    // Verify that [`load_path`] can deserialize into structs with differing names
    // as long as they have the same fields
    #[test]
    fn test_change_struct_name() -> Result<(), ConfyError> {
        with_config_path(|path| {
            #[derive(PartialEq, Default, Debug, Serialize, Deserialize)]
            struct AnotherExampleConfig {
                name: String,
                count: usize,
            }

            store_path(path, &ExampleConfig::default()).expect("store_path failed");
            let _: AnotherExampleConfig = load_path(path).expect("load_path failed");
        });

        Ok(())
    }
}
