# confy

Chat with us: [Discord](https://discord.gg/dwq4Zme)

Zero-boilerplate configuration management.

Focus on storing the right data, 
instead of worrying about how to store it.

```rust
#[macro_use]
extern crate serde_derive;

#[derive(Default, Debug, Serialize, Deserialize)]
struct MyConfig {
    version: u8,
    api_key: String,
}

fn main() -> Result<(), ::std::io::Error> {
    let cfg: MyConfig = confy::load("my-app-name")?;
    dbg!(cfg);
    Ok(())
}

Enabling the `yaml_conf` feature while disabling the default `toml_conf`
feature causes confy to use a YAML config file instead of TOML.

```
[dependencies.confy]
features = ["yaml_conf"]
default-features = false
```
