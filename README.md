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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: MyConfig = confy::load("my-app-name", None)?;
    dbg!(cfg);
    Ok(())
}
```

## Confy's feature flags

`confy` can be used with either `TOML`, `YAML`, or `RON` files.
`TOML` is the default language used with `confy` but any of the other languages can be used by enabling them with feature flags as shown below.

Note: you can only use __one__ of these features at a time, so in order to use either of the optional features you have to disable default features.

### Using YAML

To use `YAML` files with `confy` you have to make sure you have enabled the `yaml_conf` feature and disabled both `toml_conf` and `ron_conf`.

Enable the feature in `Cargo.toml`:

```toml
[dependencies.confy]
features = ["yaml_conf"]
default-features = false
```

### Using RON

For using `RON` files with `confy` you have to make sure you have enabled the `ron_conf` feature and disabled both `toml_conf` and `yaml_conf`.

Enable the feature in `Cargo.toml`:

```toml
[dependencies.confy]
features = ["ron_conf"]
default-features = false
```

## Changing Error Messages

Information about adding context to error messages can be found at [Providing Context](https://rust-cli.github.io/book/tutorial/errors.html#providing-context)

## Config File Location

`confy` uses [ProjectDirs](https://github.com/dirs-dev/directories-rs?tab=readme-ov-file#projectdirs) to store your configuration files, the common locations for those are in the `config_dir` section, below are the common OS paths:

| Linux | macOS | Windows |
| --- | --- | --- |
| `$XDG_CONFIG_HOME`/`<project_path>` or `$HOME`/.config/`<project_path>` | `$HOME`/Library/Application Support/`<project_path>` | `{FOLDERID_RoamingAppData}`/`<project_path>`/config |

Where the `<project_path>` will be `rs.$MY_APP_NAME`.

## Breaking changes

### Version 0.6.0

In this version we bumped several dependencies which have had changes with some of the default (de)serialization process:

* `serde_yaml` v0.8 -> v0.9: [v0.9 release notes](https://github.com/dtolnay/serde-yaml/releases/tag/0.9.0). There were several breaking changes to `v0.9.0` and are listed in this release tag. Especially cases where previously numbers were parsed and now return `String`. See the release notes for more details.
* `toml` v0.5 -> v0.8: [v0.8 CHANGELOG](https://github.com/toml-rs/toml/blob/main/crates/toml/CHANGELOG.md#compatibility-1). Breaking change to how tuple variants work in `toml`, from the notes: "Serialization and deserialization of tuple variants has changed from being an array to being a table with the key being the variant name and the value being the array".

### Version 0.5.0

* The base functions `load` and `store` have been added an optional parameter in the event multiples configurations are needed, or ones with different filename.
* The default configuration file is now named "default-config" instead of using the application's name. Put the second argument of `load` and `store` to be the same of the first one to keep the previous configuration file.
* It is now possible to save the configuration as `toml` or as `YAML`. The configuration's file name's extension depends on the format used.

### Version 0.4.0

Starting with version 0.4.0 the configuration file are stored in the expected place for your system. See the [`directories`] crates for more information.
Before version 0.4.0, the configuration file was written in the current directory.

[`directories`]: https://crates.io/crates/directories

## License

This work is triple-licensed under MIT, MIT/X11, or the Apache 2.0 (or any later version).
You may choose any one of these three licenses if you use this work.

`SPDX-License-Identifier: MIT OR X11 OR Apache-2.0+`
