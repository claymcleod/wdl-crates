//! Inputs parsed in from the command line.

use indexmap::IndexMap;
use regex::Regex;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::LazyLock;
use thiserror::Error;
use wdl_engine::PrimitiveValue;
use wdl_engine::Value;

pub mod file;

pub use file::InputFile;

static IDENTIFIER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // SAFETY: this is checked statically with tests to always unwrap.
    Regex::new(r"^([\w\-.]+)$").unwrap()
});

/// If a value cannot be resolved to a type, this regex is compared to the
/// value. If the regex matches, we assume the value is a string.
static ASSUME_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // SAFETY: this is checked statically with tests to always unwrap.
    Regex::new(r"^[\w ]*$").unwrap()
});

////////////////////////////////////////////////////////////////////////////////
// Error
////////////////////////////////////////////////////////////////////////////////

/// An error related to inputs.
#[derive(Error, Debug)]
pub enum Error {
    /// A file error.
    #[error("file error: {0}")]
    File(file::Error),

    /// A file was specified on the command line but not found.
    #[error("file not found: `{0}`")]
    FileNotFound(PathBuf),

    /// Encountered an invalid key-value pair.
    #[error("invalid key-value pair: {pair}\n\nreason: {reason}")]
    InvalidPair {
        /// The string-value of the pair.
        pair: String,

        /// The reason the pair was not valid.
        reason: String,
    },

    /// A deserialization error.
    #[error("unable to deserialize `{0}` as valid WDL value")]
    Deserialize(String),
}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

////////////////////////////////////////////////////////////////////////////////
// Input
////////////////////////////////////////////////////////////////////////////////

/// An input parsed from the command line.
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    /// A file.
    File(
        /// The path to the file.
        ///
        /// If this input is successfully created, the input is always
        /// guaranteed to exist at the time the inputs were processed.
        PathBuf,
    ),

    /// A key-value pair representing an input.
    Pair {
        /// The key.
        key: String,

        /// The value.
        value: Value,
    },
}

impl Input {
    /// Attempts to return a reference to the inner [`Path`].
    ///
    /// * If the input is a [`Input::File`], a reference to the inner path is
    ///   returned wrapped in [`Some`].
    /// * Otherwise, [`None`] is returned.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            Input::File(p) => Some(p.as_path()),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the inner [`PathBuf`].
    ///
    /// * If the input is a [`Input::File`], the inner path buffer is returned
    ///   wrapped in [`Some`].
    /// * Otherwise, [`None`] is returned.
    pub fn into_file(self) -> Option<PathBuf> {
        match self {
            Input::File(p) => Some(p),
            _ => None,
        }
    }

    /// Consumes `self` and returns the inner [`PathBuf`].
    ///
    /// # Panics
    ///
    /// If the input is not a [`Input::File`].
    pub fn unwrap_file(self) -> PathBuf {
        match self {
            Input::File(p) => p,
            v => panic!("{v:?} is not an `Input::File`"),
        }
    }

    /// Attempts to return a reference to the inner key-value pair.
    ///
    /// * If the input is a [`Input::Pair`], a reference to the inner key and
    ///   value is returned wrapped in [`Some`].
    /// * Otherwise, [`None`] is returned.
    pub fn as_pair(&self) -> Option<(&str, &Value)> {
        match self {
            Input::Pair { key, value } => Some((key.as_str(), value)),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the inner key-value pair.
    ///
    /// * If the input is a [`Input::Pair`], the inner key-value pair is
    ///   returned wrapped in [`Some`].
    /// * Otherwise, [`None`] is returned.
    pub fn into_pair(self) -> Option<(String, Value)> {
        match self {
            Input::Pair { key, value } => Some((key, value)),
            _ => None,
        }
    }

    /// Consumes `self` and returns the inner key-value pair.
    ///
    /// # Panics
    ///
    /// If the input is not a [`Input::Pair`].
    pub fn unwrap_pair(self) -> (String, Value) {
        match self {
            Input::Pair { key, value } => (key, value),
            v => panic!("{v:?} is not an `Input::Pair`"),
        }
    }
}

impl FromStr for Input {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if !s.contains("=") {
            let path = PathBuf::from(s);

            if !path.exists() {
                return Err(Error::FileNotFound(path));
            }

            return Ok(Input::File(path));
        }

        let parts = s.split("=").collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(Error::InvalidPair {
                pair: s.to_string(),
                reason: format!(
                    "expected exactly one equal sign (`=`), found {}",
                    parts.len() - 1,
                ),
            });
        }

        let mut parts = parts.into_iter();

        // SAFETY: we just checked to ensure that anything past this point has
        // exactly two parts, so these will always be unwrapped successfully.
        let key = parts.next().unwrap();
        let value = parts.next().unwrap();

        if !IDENTIFIER_REGEX.is_match(key) {
            return Err(Error::InvalidPair {
                pair: s.to_string(),
                reason: format!(
                    "key `{}` did not match the identifier regex (`{}`)",
                    key,
                    IDENTIFIER_REGEX.as_str()
                ),
            });
        }

        let value = value.parse::<Value>().or_else(|_| {
            if ASSUME_STRING_REGEX.is_match(value) {
                Ok(Value::Primitive(PrimitiveValue::String(
                    value.to_owned().into(),
                )))
            } else {
                Err(Error::Deserialize(value.to_owned()))
            }
        })?;

        Ok(Input::Pair {
            key: key.to_owned(),
            value,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// Inputs
////////////////////////////////////////////////////////////////////////////////

/// A set of inputs parsed from the command line and compiled on top of one
/// another.
#[derive(Clone, Debug, Default)]
pub struct Inputs(IndexMap<String, Value>);

impl Inputs {
    /// Adds an input read from the command line.
    fn add_input(&mut self, input: &str) -> Result<()> {
        match input.parse::<Input>()? {
            Input::File(path) => {
                let inputs = InputFile::read(&path).map_err(Error::File)?;
                self.extend(inputs.into_inner());
            }
            Input::Pair { key, value } => {
                self.insert(key, value);
            }
        };

        Ok(())
    }

    /// Attempts to coalesce a set of inputs into an [`Inputs`].
    pub fn coalesce<'a, T, V>(iter: T) -> Result<Self>
    where
        T: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        let mut inputs = Inputs::default();

        for input in iter {
            inputs.add_input(input.as_ref())?;
        }

        Ok(inputs)
    }

    /// Consumes `self` and returns the inner index map.
    pub fn into_inner(self) -> IndexMap<String, Value> {
        self.0
    }
}

impl Deref for Inputs {
    type Target = IndexMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_regex() {
        assert!(IDENTIFIER_REGEX.is_match("here_is-an.identifier"));
    }

    #[test]
    fn assume_string_regex() {
        assert!(ASSUME_STRING_REGEX.is_match(""));
        assert!(ASSUME_STRING_REGEX.is_match("fooBAR082"));
        assert!(ASSUME_STRING_REGEX.is_match("foo bar baz"));

        assert!(!ASSUME_STRING_REGEX.is_match("[1, a]"));
    }

    #[test]
    fn file() {
        // A valid JSON file path.
        let input = "./tests/fixtures/inputs_one.json".parse::<Input>().unwrap();
        assert!(matches!(
            input,
            Input::File(path) if path.to_str().unwrap() == "./tests/fixtures/inputs_one.json"
        ));

        // A valid YAML file path.
        let input = "./tests/fixtures/inputs_three.yml"
            .parse::<Input>()
            .unwrap();
        assert!(matches!(
            input,
            Input::File(path) if path.to_str().unwrap() == "./tests/fixtures/inputs_three.yml"
        ));

        // A missing file path.
        let err = "./tests/fixtures/missing.json"
            .parse::<Input>()
            .unwrap_err();
        assert!(matches!(
            err,
            Error::FileNotFound(path) if path.to_str().unwrap() == "./tests/fixtures/missing.json"
        ));
    }

    #[test]
    fn parse_key_value_pairs() {
        // A standard key-value pair.
        let input = r#"foo="bar""#.parse::<Input>().unwrap();
        let (key, value) = input.unwrap_pair();
        assert_eq!(key, "foo");
        assert_eq!(value.unwrap_string().as_str(), "bar");

        // A standard key-value pair.
        let input = r#"foo.bar-baz_quux="qil""#.parse::<Input>().unwrap();
        let (key, value) = input.unwrap_pair();
        assert_eq!(key, "foo.bar-baz_quux");
        assert_eq!(value.unwrap_string().as_str(), "qil");

        // An invalid identifier for the key.
        let err = r#"foo$="bar""#.parse::<Input>().unwrap_err();
        assert!(matches!(
                err,
                Error::InvalidPair {
                    pair,
                    reason
                } if pair == r#"foo$="bar""# &&
                reason == r"key `foo$` did not match the identifier regex (`^([\w\-.]+)$`)"));

        // An value that is valid despite that value not being valid as a key.
        let input = r#"foo="bar$""#.parse::<Input>().unwrap();
        let (key, value) = input.unwrap_pair();
        assert_eq!(key, "foo");
        assert_eq!(value.unwrap_string().as_str(), "bar$");
    }

    #[test]
    fn coalesce() {
        // Helper functions.
        fn check_string_value(inputs: &Inputs, key: &str, value: &str) {
            let input = inputs.get(key).unwrap();
            assert_eq!(input.as_string().unwrap().as_str(), value);
        }

        fn check_float_value(inputs: &Inputs, key: &str, value: f64) {
            let input = inputs.get(key).unwrap();
            assert_eq!(input.as_float().unwrap(), value);
        }

        fn check_boolean_value(inputs: &Inputs, key: &str, value: bool) {
            let input = inputs.get(key).unwrap();
            assert_eq!(input.as_boolean().unwrap(), value);
        }

        fn check_integer_value(inputs: &Inputs, key: &str, value: i64) {
            let input = inputs.get(key).unwrap();
            assert_eq!(input.as_integer().unwrap(), value);
        }

        // The standard coalescing order.
        let inputs = Inputs::coalesce([
            "./tests/fixtures/inputs_one.json",
            "./tests/fixtures/inputs_two.json",
            "./tests/fixtures/inputs_three.yml",
        ])
        .unwrap();

        assert_eq!(inputs.len(), 5);
        check_string_value(&inputs, "foo", "bar");
        check_float_value(&inputs, "baz", 128.0);
        check_string_value(&inputs, "quux", "qil");
        check_string_value(&inputs, "new", "foobarbaz");
        check_string_value(&inputs, "new_two", "bazbarfoo");

        // The opposite coalescing order.
        let inputs = Inputs::coalesce([
            "./tests/fixtures/inputs_three.yml",
            "./tests/fixtures/inputs_two.json",
            "./tests/fixtures/inputs_one.json",
        ])
        .unwrap();

        assert_eq!(inputs.len(), 5);
        check_string_value(&inputs, "foo", "bar");
        check_float_value(&inputs, "baz", 42.0);
        check_string_value(&inputs, "quux", "qil");
        check_string_value(&inputs, "new", "foobarbaz");
        check_string_value(&inputs, "new_two", "bazbarfoo");

        // An example with some random key-value pairs thrown in.
        let inputs = Inputs::coalesce([
            r#"sandwich=-100"#,
            "./tests/fixtures/inputs_one.json",
            "./tests/fixtures/inputs_two.json",
            r#"quux="jacks""#,
            "./tests/fixtures/inputs_three.yml",
            r#"baz=false"#,
        ])
        .unwrap();

        assert_eq!(inputs.len(), 6);
        check_string_value(&inputs, "foo", "bar");
        check_boolean_value(&inputs, "baz", false);
        check_string_value(&inputs, "quux", "jacks");
        check_string_value(&inputs, "new", "foobarbaz");
        check_string_value(&inputs, "new_two", "bazbarfoo");
        check_integer_value(&inputs, "sandwich", -100);

        // An invalid key-value pair.
        let error =
            Inputs::coalesce(["./tests/fixtures/inputs_one.json", "foo=baz#bar"]).unwrap_err();
        assert!(matches!(
            error,
            Error::Deserialize(value) if value == "baz#bar"
        ));

        // A missing file.
        let error = Inputs::coalesce([
            "./tests/fixtures/inputs_one.json",
            "./tests/fixtures/inputs_two.json",
            "./tests/fixtures/inputs_three.yml",
            "./tests/fixtures/missing.json",
        ])
        .unwrap_err();
        assert!(matches!(
                error,
                Error::FileNotFound(path) if path.to_str().unwrap() == "./tests/fixtures/missing.json"));
    }
}
