//! This is an example of how this crate could be usable

#[macro_use] extern crate confy;

#[derive(Confy)]
struct MyConfig {
    /* ... some fields ... */
}


fn main() {

    // Creates a folder for your app called `my_app` and the
    // actual config file is derived from the struct name
    //
    // The path is also dependant on the OS and all of this happens
    // completely transparently
    let cfg: MyConfig = match confy::load("my_app") {
        Some(cfg) => cfg,
        None => confy::create(("my_app", MyConfig {
            /* ... initialise default ... */
        }),
    };

    // Here `cfg` is just a usable struct
    // ...

    // At some point you can save/ overwrite your config
    confy::save(("my_app", cfg).unwrap(); // Returns Result
}