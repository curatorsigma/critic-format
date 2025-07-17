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
            // we consider an abbreviations language to be that of the expansion
            // not that of the surface form. e.g. the πιπι for יהוה abbreviation is part of
            // hbo-Hebr text and therefor should not have its language determined by the fact that
            // πιπι is in grc
            | Self::Abbreviation(Abbreviation { expansion_lang: x, .. }) => Some(x),
        }
    }
}
pub trait FromTypeLangAndContent {
    fn from_type_lang_and_content(block_type: BlockType, lang: String, content: String) -> Self;

    #[must_use]
    fn from_type_and_lang(block_type: BlockType, lang: String) -> Self
    where
        Self: Sized,
    {
        Self::from_type_lang_and_content(block_type, lang, String::default())
    }
}

/// Dataless enum for block types
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum BlockType {
    /// A break in the text - line or column break
    Break,
    /// A lacuna in the manuscript
    Lacuna,
    /// An anchor - the beginning of a verse
    Anchor,
    // Normal unmarked text
    Text,
    /// A correction in the manuscript - where one scribal hand has overwritte / struck through / .. a text that was present earlier
    Correction,
    // A part of text that is damaged but still legible
    Uncertain,
    // An expanded abbreviation
    Abbreviation,
}
impl FromTypeLangAndContent for Block {
    fn from_type_lang_and_content(block_type: BlockType, lang: String, content: String) -> Self {
        match block_type {
            BlockType::Text => Self::Text(Paragraph { lang, content }),
            BlockType::Abbreviation => Self::Abbreviation(Abbreviation {
                surface: content.clone(),
                surface_lang: lang.clone(),
                expansion: content,
                expansion_lang: lang,
            }),
            BlockType::Break => Self::Break(BreakType::default()),
            BlockType::Lacuna => Self::Lacuna(Lacuna {
                unit: ExtentUnit::default(),
                n: 1,
                reason: String::default(),
                cert: None,
            }),
            BlockType::Anchor => Self::Anchor(Anchor::default()),
            BlockType::Uncertain => Self::Uncertain(Uncertain {
                lang,
                cert: None,
                agent: String::default(),
                content: content.to_string(),
            }),
            BlockType::Correction => Self::Correction(Correction {
                lang: lang.clone(),
                versions: vec![Version {
                    lang,
                    hand: None,
                    content,
                }],
            }),
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
/// Default for user facing code
impl Default for BreakType {
    fn default() -> Self {
        Self::Line
    }
}
/// Convert Line => Line, Column => Column and reject everything else (case sensitive)
impl core::str::FromStr for BreakType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Line" => Ok(Self::Line),
            "Column" => Ok(Self::Column),
            _ => Err(()),
        }
    }
}
impl BreakType {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Line => "Line",
            Self::Column => "Column",
        }
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
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Uncertain {
    /// The language of this uncertain passage
    pub lang: String,
    /// The certainty the transcriber assigns to the reconstruction of the damaged text
    pub cert: Option<String>,
    /// The cause of damage
    pub agent: String,
    /// The reproduction of the damaged text
    pub content: String,
}

/// An expanded abbreviation.
///
/// NOTE:
/// it is common to see abbreviations such as πιπι for יהוה
/// We signal the differing languages by having two lang attributes here: one for the surface form,
/// one for the expanded form.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Abbreviation {
    /// The language of the surface form of this abbreviation
    pub surface_lang: String,
    /// The surface form (the abbreviation) present in the manuscript
    pub surface: String,
    /// The language of the expanded form of this abbreviation
    pub expansion_lang: String,
    /// The expanded form supplied by the transcriber
    pub expansion: String,
}

pub type ExtentUnit = normalized::ExtentUnit;
