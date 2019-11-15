# confy

Chat with us: [Discord](https://discord.gg/dwq4Zme)

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


