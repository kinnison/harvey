//! Harvey Slideware
//!

// Until such time as we get a new marked-yaml with better
// error struct sizes, we need this:
#![allow(clippy::result_large_err)]
// Expect everyting to be documented
#![deny(missing_docs)]

pub mod formats;
pub mod resources;
pub mod yaml;
