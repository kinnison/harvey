//! Data file formats for Harvey
//!
//! The types in here are for parsing the various data formats
//! which Harvey has, including [`DeckFile`]s, [`SlideFile`]s,
//! and within the latter [`SlideContent`]s, and [`SlideMetadata`].

mod deckfile;
mod slides;

#[doc(inline)]
pub use deckfile::DeckFile;

#[doc(inline)]
pub use slides::SlideMetadata;

#[doc(inline)]
pub use slides::SlideFile;

#[doc(inline)]
pub use slides::SlideContent;
