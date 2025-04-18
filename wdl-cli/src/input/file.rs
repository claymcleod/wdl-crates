//! Input files parsed in from the command line.

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use wdl_engine::CompoundValue;
use wdl_engine::Object;
use wdl_engine::Value;

use crate::Inputs;

/// An error related to a input file.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error.
    #[error(transparent)]
    Io(std::io::Error),

    /// The input file did not contain a map at the root.
    #[error("input file `{0}` did not contain a map at the root")]
    NonMapRoot(PathBuf),

    /// Neither JSON nor YAML could be parsed from the provided path.
    #[error("file `{0}` is neither a valid JSON nor a valid YAML file")]
    UnsupportedFormat(PathBuf),
}

/// An input file for a WDL run.
pub struct InputFile;

impl InputFile {
    /// Reads a input file.
    ///
    /// The file is attempted to be parsed as JSON and, if that fails, then
    /// YAML. If either is succesful, the inner value map is returned wrapped in
    /// [`Ok`]. If neither is able to be parsed, an [`Error::UnsupportedFormat`] is returned.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Inputs, Error> {
        let path = path.as_ref();
        let content: String = std::fs::read_to_string(path).map_err(Error::Io)?;

        fn coerce_object_to_inputs(object: Object) -> Result<Inputs, Error> {
            let mut inputs = Inputs::default();

            for (key, value) in object.iter() {
                inputs.insert(key.to_owned(), value.clone());
                dbg!(&value);
            }

            Ok(inputs)
        }

        if let Ok(value) = serde_json::from_str::<Value>(&content) {
            return match value {
                Value::Compound(CompoundValue::Object(object)) => coerce_object_to_inputs(object),
                _ => Err(Error::NonMapRoot(path.to_path_buf())),
            };
        }

        if let Ok(value) = serde_yaml_ng::from_str::<Value>(&content) {
            return match value {
                Value::Compound(CompoundValue::Object(object)) => coerce_object_to_inputs(object),
                _ => Err(Error::NonMapRoot(path.to_path_buf())),
            };
        }

        Err(Error::UnsupportedFormat(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonmap_root() {
        // A JSON file that does not have a map at the root.
        let err = InputFile::read(Path::new("./tests/fixtures/nonmap_inputs.json")).unwrap_err();
        assert!(matches!(
            err,
            Error::NonMapRoot(path) if path.to_str().unwrap() == "./tests/fixtures/nonmap_inputs.json"
        ));

        // A YML file that does not have a map at the root.
        let err = InputFile::read(Path::new("./tests/fixtures/nonmap_inputs.yml")).unwrap_err();
        assert!(matches!(
            err,
            Error::NonMapRoot(path) if path.to_str().unwrap() == "./tests/fixtures/nonmap_inputs.yml"
        ));
    }
}
