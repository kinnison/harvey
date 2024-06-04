//! YAML related works
//!
//! All YAML loaded, or deserialised, should be processed via this file's functions.
//! This will ensure that we always have a logged source for the files.

use std::{
    path::{Path, PathBuf},
    str::Utf8Error,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use marked_yaml::{
    from_yaml_with_options, parse_yaml_with_options, FromYamlError, LoadError, LoaderOptions, Node,
};
use serde::de::DeserializeOwned;
use thiserror::Error;

/// A source of YAML data
pub enum YamlSource {
    /// Directly loaded from a file on disk (eg. a deck file or an overridden file)
    DiskFile(PathBuf),
    /// Loaded from an embedded resource
    Resource(String),
    /// Loaded as slide metadata from parsing the given slide file.
    ///
    /// The first `usize` is the slide number in the file and the second
    /// is the line offset for error reporting.
    Slide(Arc<Path>, usize, usize),
}

static SOURCES: OnceLock<Mutex<Vec<YamlSource>>> = OnceLock::new();

fn sources() -> MutexGuard<'static, Vec<YamlSource>> {
    SOURCES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("Something bad happened in sources()")
}

/// A load error when trying to read YAML
#[derive(Error, Debug)]
pub enum YAMLLoadError {
    /// Something went wrong reading IO
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// The content of the input was not valid UTF8,
    #[error(transparent)]
    UTF8(#[from] Utf8Error),
    /// Something went wrong deserialising the content
    #[error(transparent)]
    FromYaml(#[from] FromYamlError),
}

/// Load some YAML from disk
///
/// This loads YAML from disk, deserialising it as appropriate.  We log the source
/// into the global [`SOURCES`] from where we can find it later.
pub fn from_file<T>(path: impl AsRef<Path>) -> Result<T, YAMLLoadError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref().to_owned();
    let content = std::fs::read_to_string(&path)?;

    Ok(from_source(YamlSource::DiskFile(path), &content)?)
}

/// Load some YAML from a built-in resource
pub fn from_resource<T>(resname: &str, content: &str) -> Result<T, FromYamlError>
where
    T: DeserializeOwned,
{
    from_source(YamlSource::Resource(resname.to_string()), content)
}

/// Load some YAML specified as slide metadata
pub fn from_slide<T>(
    slidefile: Arc<Path>,
    slidenr: usize,
    lineoffset: usize,
    content: &str,
) -> Result<T, FromYamlError>
where
    T: DeserializeOwned,
{
    from_source(YamlSource::Slide(slidefile, slidenr, lineoffset), content)
}

/// Load some YAML from a named source
pub fn from_source<T>(source: YamlSource, content: &str) -> Result<T, FromYamlError>
where
    T: DeserializeOwned,
{
    let options = LoaderOptions {
        error_on_duplicate_keys: true,
    };
    let mut sources = sources();

    let ret: T = from_yaml_with_options(sources.len(), content, options)?;

    sources.push(source);

    Ok(ret)
}

/// Load some YAML from a named source, without deserialising
pub fn node_from_source(source: YamlSource, content: &str) -> Result<Node, LoadError> {
    let options = LoaderOptions {
        error_on_duplicate_keys: true,
    };
    let mut sources = sources();

    let ret = parse_yaml_with_options(sources.len(), content, options)?;

    sources.push(source);

    Ok(ret)
}
