//! Definition for the streamed representation of transcriptions.
//!
//! The text is hierarchically ordered into columns and lines in [`schema`](crate::schema) and
//! [`normalized`](crate::normalized). However, the editor in
//! [`critic`](https://github.com/curatorsigma/critic) has individual blocks as top-level elements,
//! with column and line breaks being blocks themselves. These types represents the data as seen in
//! the editor.

use serde::{Deserialize, Serialize};

use crate::normalized;

/// An entire manuscript with its content streamed
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Manuscript {
    /// The header with any meta-information
    pub meta: Meta,
    /// the actual text in individual blocks
    pub content: Vec<Block>,
}

pub type Meta = normalized::Meta;

/// A block in the editor, without the associated language
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Block {
    /// A break in the text - line or column break
    Break(BreakType),
    /// A lacuna in the manuscript
    Lacuna(Lacuna),
    /// An anchor - the beginning of a verse
    Anchor(Anchor),
    // Normal unmarked text
    Text(Paragraph),
    /// A correction in the manuscript - where one scribal hand has overwritte / struck through / .. a text that was present earlier
    Correction(Correction),
    // A part of text that is damaged but still legible
    Uncertain(Uncertain),
    // An expanded abbreviation
    Abbreviation(Abbreviation),
}
impl Block {
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        match self {
            Self::Break(_) | Self::Lacuna(_) | Self::Anchor(_) => None,
            Self::Text(Paragraph { lang: x, .. })
            | Self::Correction(Correction { lang: x, .. })
            | Self::Uncertain(Uncertain { lang: x, .. })
            | Self::Abbreviation(Abbreviation { lang: x, .. }) => Some(x),
        }
    }
}

/// The different types of Break that can occur in a critic-TEI file
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum BreakType {
    /// Linebreak
    Line,
    /// Columnbreak
    Column,
}
/// Default is a linebreak
impl Default for BreakType {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Paragraph {
    pub lang: String,
    pub content: String,
}

pub type Lacuna = normalized::Lacuna;
pub type Anchor = normalized::Anchor;

/// An ancient correction.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Correction {
    /// The language of this correction
    pub lang: String,
    /// A list of different readings. Each form this manuscript had at one point should get its own
    /// reading and be written out in its entirety here.
    pub versions: Vec<Version>,
}

/// An individual reading (version) inside a correction.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Version {
    /// The language of this version
    pub lang: String,
    /// The scribal hand responsible for this reading
    ///
    /// The different hands should be explained in the [`<handDesc>`](crate::schema::HandDesc) in the header.
    pub hand: Option<String>,
    /// The actual text of this reading
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Uncertain {
    /// The language of this uncertain passage
    pub lang: String,
    /// The certainty the transcriber assigns to the reconstruction of the damaged text
    pub cert: String,
    /// The cause of damage
    pub agent: String,
    /// The reproduction of the damaged text
    pub text: String,
}

/// An expanded abbreviation.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Abbreviation {
    /// The language of this abbreviation
    pub lang: String,
    /// The surface form (the abbreviation) present in the manuscript
    pub surface: String,
    /// The expanded form supplied by the transcriber
    pub expansion: String,
}

pub type ExtentUnit = normalized::ExtentUnit;
