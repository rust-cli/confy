//! The most simplest examples of how to use confy

extern crate confy;

#[macro_use]
extern crate serde_derive;

use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
struct ConfyConfig {
    name: String,
    comfy: bool,
    foo: i64,
}

impl Default for ConfyConfig {
    fn default() -> Self {
        ConfyConfig {
            name: "Unknown".to_string(),
            comfy: true,
            foo: 42,
        }
    }
}

fn main() -> Result<(), confy::ConfyError> {
    let cfg: ConfyConfig = confy::load("confy_simple_app", None)?;
    let file = confy::get_configuration_file_path("confy_simple_app", None)?;
    println!("The configuration file path is: {:#?}", file);
    println!("The configuration is:");
    println!("{:#?}", cfg);
    println!("The wrote toml file content is:");
    let mut content = String::new();
    std::fs::File::open(&file)
        .expect("Failed to open toml configuration file.")
        .read_to_string(&mut content)
        .expect("Failed to read toml configuration file.");
    println!("{}", content);
    let cfg = ConfyConfig {
        name: "Test".to_string(),
        ..cfg
    };
    confy::store("confy_simple_app",None, &cfg)?;
    println!("The updated toml file content is:");
    let mut content = String::new();
    std::fs::File::open(&file)
        .expect("Failed to open toml configuration file.")
        .read_to_string(&mut content)
        .expect("Failed to read toml configuration file.");
    println!("{}", content);
    let _cfg = ConfyConfig {
        name: "Test".to_string(),
        ..cfg
    };
    std::fs::remove_dir_all(file.parent().unwrap())
        .expect("Failed to remove directory");
    Ok(())
}
