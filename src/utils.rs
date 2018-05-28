//! Some storage utilities

use std::io::Error as IoError;
use serde::Serialize;
use std::{fs::File, io::Read};

/// A folder scaffolding utility which reports if errors occured
pub(crate) fn scaffold_directories() -> Result<(), IoError> {
    Ok(())
} 

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