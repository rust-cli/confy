# confy

[![Join the chat at https://gitter.im/rust-clique/confy](https://badges.gitter.im/rust-clique/confy.svg)](https://gitter.im/rust-clique/confy?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Zero-boilerplate configuration management.

Focus on storing the right data, 
instead of worrying about how to store it.

```rust
#[derive(Serialize, Deserialize)]
struct MyConfig {
    version: u8,
    api_key: String,
}

/// `MyConfig` implements `Default`
impl ::std::ops::Default for MyConfig {
    fn default() -> Self { Self { version: 0, api_key: "".into() } }
}

fn main() -> Result<(), ::std::io::Error> {
    let cfg = confy::load("my-app-name")?;
}
```


