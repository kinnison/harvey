//! Deck file format
//!

use std::{borrow::Cow, collections::HashMap, path::Path};

use marked_yaml::Spanned;
use serde::Deserialize;
use tera::{Map, Value};

use crate::yaml::{from_file, YAMLLoadError};

use super::SlideMetadata;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
/// A Deck file
///
/// The deck is the top level file for harvey slides.  It defines, at bare minimum, the
/// slide file(s) to load to build the deck.
pub struct DeckFile {
    markdown: Option<Markdown>,
    context: Option<Value>,
    meta: Option<SlideMetadata>,
    #[serde(default)]
    styles: Vec<Spanned<String>>,
    #[serde(default)]
    scripts: Vec<Spanned<String>>,
    #[serde(default)]
    template_path: Vec<Spanned<String>>,
    slides: Vec<Spanned<String>>,
    #[serde(default)]
    tree_sitter_highlight: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The markdown configuration for the deck
pub struct Markdown {
    pub blockquote: Option<MarkdownBlockQuote>,
    pub code_block_prefix: Option<Spanned<String>>,
    pub code_block_focus: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The blockquote markdown configuration for the deck
pub struct MarkdownBlockQuote {
    pub note: Option<Spanned<String>>,
    pub tip: Option<Spanned<String>>,
    pub important: Option<Spanned<String>>,
    pub warning: Option<Spanned<String>>,
    pub caution: Option<Spanned<String>>,
}

impl DeckFile {
    /// Load a deck from disk, logging the YAML into the global YAML context
    pub fn from_file<P>(path: impl AsRef<Path>) -> Result<Self, YAMLLoadError> {
        from_file(path)
    }

    /// Perform merges where `other` is considered the default values
    pub fn merge_from(&mut self, _other: &DeckFile) {
        todo!()
    }

    /// The style resources
    pub fn styles(&self) -> &[Spanned<String>] {
        &self.styles
    }

    /// The script resources
    pub fn scripts(&self) -> &[Spanned<String>] {
        &self.scripts
    }

    /// Template paths
    pub fn template_path(&self) -> &[Spanned<String>] {
        &self.template_path
    }

    /// Slides
    pub fn slides(&self) -> &[Spanned<String>] {
        &self.slides
    }

    /// The markdown metadata
    pub fn markdown(&self) -> Option<&Markdown> {
        self.markdown.as_ref()
    }

    /// Context
    pub fn context(&self) -> Cow<'_, Value> {
        self.context
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Value::Object(Map::new())))
    }

    /// Metadata
    pub fn meta(&self) -> Option<&SlideMetadata> {
        self.meta.as_ref()
    }

    /// Tree sitter highlight rules
    ///
    /// The iterator returned is pairs of (highlight-name, css-class-name)
    pub fn tree_sitter_highlight(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.tree_sitter_highlight
            .iter()
            .flat_map(|map| map.iter().map(|(k, v)| (v.as_str(), k.as_str())))
    }
}
