# confy

[![Join the chat at https://gitter.im/rust-clique/confy](https://badges.gitter.im/rust-clique/confy.svg)](https://gitter.im/rust-clique/confy?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

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
```
