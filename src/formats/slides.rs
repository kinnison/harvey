//! Slide data formats
//!

use std::fmt::Write;
use std::{path::Path, sync::Arc};

use marked_yaml::{LoadError, Node, Spanned};
use serde::{de::Visitor, Deserialize};
use thiserror::Error;

use crate::yaml::{self, YamlSource};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Metadata associated with slides
///
/// Note, this isn't full slide metadata, which is an arbitrary dictionary,
/// but rather the specific metadata which Harvey needs to work upon
pub struct SlideMetadata {
    /// The name to put into the context containing the slide content
    pub content_name: Option<Spanned<String>>,
    /// The name to put into the context containing the subslide elements
    pub content_list: Option<Spanned<String>>,
    /// The name of the template to use if none specified
    pub default_template: Option<Spanned<String>>,
    /// Which slide metadata values to inherit from slide to slide
    #[serde(default)]
    pub inherit: Vec<Spanned<String>>,
    /// Which slide metadata values are required
    #[serde(default)]
    pub require: Vec<Spanned<String>>,
    /// Which metadata value names are denied
    #[serde(default)]
    pub deny: Vec<Spanned<String>>,
    /// The screen ratio
    #[serde(default)]
    pub ratio: Option<SlideRatio>,
}

/// The ratio to use for the slide deck
pub struct SlideRatio {
    pub width: usize,
    pub height: usize,
}

impl<'de> Deserialize<'de> for SlideRatio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(RatioVisitor)
    }
}

struct RatioVisitor;
impl<'de> Visitor<'de> for RatioVisitor {
    type Value = SlideRatio;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a pair of positive integers separated by a colon")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((width, height)) = v.split_once(':') {
            let width: usize = width.parse().map_err(E::custom)?;
            let height: usize = height.parse().map_err(E::custom)?;
            Ok(SlideRatio { width, height })
        } else {
            Err(E::custom(format!("Expected X:Y, got {}", v)))
        }
    }
}

/*****************************/

/// Errors which can happen while loading slides
#[derive(Debug, Error)]
pub enum SlideLoadError {
    /// IO error
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// The initial delimiter is missing from the file, so effectively no slides are present
    #[error("Missing initial delimiter.  Slide files must start with ---")]
    MissingInitialDelimiter,
    /// The metadata started at the given line number (1-indexed) is incomplete
    #[error("Incomplete metadata found at line {0}")]
    IncompleteMetadata(usize),
    /// The metadata started at the given line number (1-indexed) is bad YAML
    #[error("Bad yaml found at line {0}: {1}")]
    BadMetadata(usize, LoadError),
}

/// A file of slides
///
#[derive(Debug)]
pub struct SlideFile {
    fname: Arc<Path>,
    slides: Vec<SlideContent>,
}

/// A single slide
#[derive(Debug)]
pub struct SlideContent {
    meta: Node,
    lineno: usize,
    parts: Vec<String>,
    notes: String,
}

impl SlideContent {
    /// Get the raw metadata
    pub fn meta_raw(&self) -> &Node {
        &self.meta
    }

    /// Get the line number on which this slide starts
    ///
    /// This is the first line of the metadata (or content if there is no metadata)
    pub fn lineno(&self) -> usize {
        self.lineno
    }

    /// Get the raw parts
    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    /// Get the slide notes
    pub fn notes(&self) -> &str {
        &self.notes
    }
}

impl SlideFile {
    /// The name of the slide file
    pub fn fname(&self) -> Arc<Path> {
        Arc::clone(&self.fname)
    }

    /// The slides in this file
    pub fn slides(&self) -> &[SlideContent] {
        &self.slides
    }

    /// Load a slide file from disk and parse it into memory
    ///
    /// The loading is done as "kindly" as possible, but if something
    /// is very broken then we refuse to continue.  Our return value
    /// is either the loaded slide file, or as many errors as we can
    /// usefully report.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Vec<SlideLoadError>> {
        let mut errs = Vec::new();
        let fname: Arc<Path> = path.as_ref().into();
        let mut ret = SlideFile {
            fname,
            slides: Vec::new(),
        };

        let text = std::fs::read_to_string(ret.fname.as_ref()).map_err(|e| vec![e.into()])?;

        enum ParseMode {
            Initial,
            Metadata(usize, &'static str, String),
            CapturingSlide(SlideContent),
            CapturingNotes(SlideContent),
            Aborting,
        }
        use ParseMode::*;

        let mut mode = Initial;
        for (raw_lineofs, line) in text.lines().enumerate() {
            mode = match mode {
                Initial => {
                    if !line.chars().all(|c| c == '-') {
                        break;
                    }
                    if line.len() > 3 {
                        Metadata(raw_lineofs, "...", String::new())
                    } else {
                        Metadata(raw_lineofs, "", String::new())
                    }
                }
                Metadata(ofs, delim, mut raw_meta) => {
                    if line == delim {
                        let source =
                            YamlSource::Slide(ret.fname(), ret.slides.len() + 1, raw_lineofs);
                        match yaml::node_from_source(source, &raw_meta) {
                            Ok(node) => CapturingSlide(SlideContent {
                                meta: node,
                                lineno: ofs + 1,
                                parts: vec![String::new()],
                                notes: String::new(),
                            }),
                            Err(e) => {
                                errs.push(SlideLoadError::BadMetadata(ofs + 1, e));
                                Aborting
                            }
                        }
                    } else {
                        raw_meta.push_str(line);
                        raw_meta.push('\n');
                        Metadata(ofs, delim, raw_meta)
                    }
                }
                CapturingSlide(mut slide) => {
                    if line == "***" {
                        slide.parts.push(String::new());
                        CapturingSlide(slide)
                    } else if line == "???" {
                        CapturingNotes(slide)
                    } else if line.chars().all(|c| c == '-') {
                        ret.slides.push(slide);
                        if line.len() > 3 {
                            Metadata(raw_lineofs, "...", String::new())
                        } else {
                            Metadata(raw_lineofs, "", String::new())
                        }
                    } else {
                        // Unwrap is fine since there's always at least one part
                        writeln!(slide.parts.last_mut().unwrap(), "{}", line)
                            .expect("Out of memory?");
                        CapturingSlide(slide)
                    }
                }
                CapturingNotes(mut slide) => {
                    if line.chars().all(|c| c == '-') {
                        ret.slides.push(slide);
                        if line.len() > 3 {
                            Metadata(raw_lineofs, "...", String::new())
                        } else {
                            Metadata(raw_lineofs, "", String::new())
                        }
                    } else {
                        writeln!(&mut slide.notes, "{}", line).expect("Out of memory?");
                        CapturingNotes(slide)
                    }
                }
                Aborting => {
                    if line.chars().all(|c| c == '-') {
                        if line.len() > 3 {
                            Metadata(raw_lineofs, "...", String::new())
                        } else {
                            Metadata(raw_lineofs, "", String::new())
                        }
                    } else {
                        Aborting
                    }
                }
            };
        }

        match mode {
            Initial => errs.push(SlideLoadError::MissingInitialDelimiter),
            Metadata(ofs, _, _) => errs.push(SlideLoadError::IncompleteMetadata(ofs + 1)),
            CapturingSlide(slide) | CapturingNotes(slide) => {
                ret.slides.push(slide);
            }
            Aborting => {}
        }

        if errs.is_empty() {
            Ok(ret)
        } else {
            Err(errs)
        }
    }
}
