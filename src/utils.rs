//! Some storage utilities

use std::fs::File;
use std::io::{Read, Error as IoError};

pub trait CheckedStringRead {
    fn get_string(&mut self) -> Result<String, IoError>;
}

impl CheckedStringRead for File {
    fn get_string(&mut self) -> Result<String, IoError> {
        let mut s = String::new();
        self.read_to_string(&mut s)?;
        Ok(s)
    }
}
