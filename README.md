# confy

[![crates.io](https://img.shields.io/crates/v/confy)](https://crates.io/crates/confy)
[![docs.rs](https://img.shields.io/docsrs/confy)](https://docs.rs/confy/)
[![Discord](https://img.shields.io/badge/chat-Discord-informational)](https://discord.gg/dwq4Zme)

Zero-boilerplate configuration management.

Focus on storing the right data, instead of worrying about how or where to store it.

```rust
use serde_derive::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
struct MyConfig {
    version: u8,
    api_key: String,
}

fn main() -> Result<(), ::std::io::Error> {
    let cfg: MyConfig = confy::load("my-app-name", None)?;
    dbg!(cfg);
    Ok(())
}
```

## Confy's feature flags
Confy can be used with either `TOML`, `YAML`, or `RON` files.
`TOML` is the default language used with confy but any of the other languages can be used by enabling them with feature flags as shown below.

Note: you can only use __one__ of these features at a time, so in order to use either of the optional features you have to disable default features.

### Using yaml
To use YAML files with confy you have to make sure you have enabled the `yaml_conf` feature and disabled both `toml_conf` and `ron_conf`.

Enable the feature in `Cargo.toml`:
```toml
[dependencies.confy]
features = ["yaml_conf"]
default-features = false
```

### Using ron
For using RON files with confy you have to make sure you have enabled the `ron_conf` feature and disabled both `toml_conf` and `yaml_conf`.

Enable the feature in `Cargo.toml`:
```toml
[dependencies.confy]
features = ["ron_conf"]
default-features = false
```

## Breaking changes
### Version 0.5.0
* The base functions `load` and `store` have been added an optional parameter in the event multiples configurations are needed, or ones with different filename.
* The default configuration file is now named "default-config" instead of using the application's name. Put the second argument of `load` and `store` to be the same of the first one to keep the previous configuration file.
* It is now possible to save the configuration as toml or as yaml. The configuration's file name's extension depends on the format used.

### Version 0.4.0
Starting with version 0.4.0 the configuration file are stored in the expected place for your system. See the [`directories`] crates for more information.
Before version 0.4.0, the configuration file was written in the current directory.

[`directories`]: https://crates.io/crates/directories
[`directories-next`]: https://crates.io/crates/directories-next

