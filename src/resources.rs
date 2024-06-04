//! Resources for Harvey
//!
//! Resources are a combination of any file built in, and also
//! the files which can be accessed via the template paths.
//!
//! When a resource is looked for, we first try and load from
//! the specified paths in reverse order.  If we cannot find it
//! on disk then we look to load it from an internal resource.

use std::{
    borrow::Cow,
    io::{self},
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};

use rust_embed::Embed;
use serde::de::DeserializeOwned;

use crate::yaml::{self, YAMLLoadError, YamlSource};

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/assets"]
#[include = "*.html"]
#[include = "*.sass"]
#[include = "*.scss"]
#[include = "*.css"]
#[include = "*.inc"]
#[include = "*.js"]
#[include = "*.macro"]
struct Resources;

static RESOURCE_PATHS: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

fn paths() -> MutexGuard<'static, Vec<PathBuf>> {
    RESOURCE_PATHS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("Something went wrong in resource::paths()")
}

/// Retrieve a resource by name
pub fn get(resname: &str) -> io::Result<(Option<PathBuf>, Cow<'static, [u8]>)> {
    for path in paths().iter().rev() {
        match std::fs::read(path) {
            Ok(content) => return Ok((Some(path.to_path_buf()), content.into())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    // Wasn't in the resource paths, check internal resource
    Resources::get(resname)
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "resource not found",
        ))
        .map(|efile| (None, efile.data))
}

/// Load YAML from a resource, by name
pub fn get_yaml<T>(resname: &str) -> Result<T, YAMLLoadError>
where
    T: DeserializeOwned,
{
    let (fname, content) = get(resname)?;
    let source = fname
        .map(YamlSource::DiskFile)
        .unwrap_or_else(|| YamlSource::Resource(resname.to_string()));
    let content = std::str::from_utf8(&content)?;
    Ok(yaml::from_source(source, content)?)
}
