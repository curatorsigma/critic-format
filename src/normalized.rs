//! Rust types that map more directly to the semantics of our data.
//!
//! We (de-)serialize with the types in [`schema`](crate::schema) and then convert them to these nicer datatypes
//! that do not have to map so closely to the xml format.

/// An entire manuscript with normalized meta and content
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Manuscript {
    /// The header with any meta-information
    pub meta: Meta,
    /// the actual text
    pub text: Text,
}

/// TEI fileDesc element - descripbes this file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Meta {
    pub name: String,
    /// number of the page (or folio/r-v)
    pub page_nr: String,
    /// Title of this manuscript
    pub title: String,
    pub institution: Option<String>,
    pub collection: Option<String>,
    pub hand_desc: String,
    pub script_desc: String,
}

/// The entire transcribed text body
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Text {
    /// the default language for text in this manuscript
    pub lang: String,
    /// The columns present in this text
    pub columns: Vec<Column>,
}

/// A complete column in the manuscript.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Column {
    /// The default language of text in this column
    pub lang: Option<String>,
    /// the column number
    pub n: i32,
    /// The lines in this column
    pub lines: Vec<Line>,
}

/// A complete line in the manuscript.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Line {
    /// The default language of text in this line
    pub lang: Option<String>,
    /// the line number
    pub n: i32,
    /// The actual text elements contained in this line
    pub blocks: Vec<InlineBlock>,
}

/// A block of text or marked up text.
///
/// This maps to the Blocks (with actual content) that are editable in
/// [critic](https://github.com/curatorsigma/critic).
/// These are atomic units of text, that NEVER overlap linebreaks.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InlineBlock {
    /// A lacuna in the manuscript
    Lacuna(Lacuna),
    /// An anchor - the beginning of a verse
    Anchor(Anchor),
    /// A correction in the manuscript - where one scribal hand has overwritte / struck through / .. a text that was present earlier
    Correction(Correction),
    Text(Paragraph),
    Uncertain(Uncertain),
    Abbreviation(Abbreviation),
}
impl InlineBlock {
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        match self {
            Self::Lacuna(_) | Self::Anchor(_) => None,
            Self::Correction(x) => x.lang.as_deref(),
            Self::Text(x) => x.lang.as_deref(),
            Self::Uncertain(x) => x.lang.as_deref(),
            Self::Abbreviation(x) => x.lang.as_deref(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Paragraph {
    pub lang: Option<String>,
    pub content: String,
}

/// The beginning of a verse.
///
/// This could either be marked in the manuscript (Sof Passuq, verse number etc.) or supplied from
/// other manuscripts if it is certain which verse begins at this point.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Anchor {
    /// The ID of this verse.
    ///
    /// MUST be `A_V_{versification-theme-shorthand}_{verse-number}`
    pub anchor_id: String,
    /// MUST be `{versification-theme-long-form}`
    pub anchor_type: String,
}

pub type Uncertain = crate::schema::Damage;
pub type Abbreviation = crate::schema::Choice;

/// An ancient correction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Correction {
    /// The language set on the `<app>` element
    pub lang: Option<String>,
    /// A list of different readings. Each form this manuscript had at one point should get its own
    /// reading and be written out in its entirety here.
    pub versions: Vec<Version>,
}

/// An individual reading (version) inside a correction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Version {
    /// the language set on the `<rdg>`
    pub lang: Option<String>,
    /// The scribal hand responsible for this reading
    ///
    /// The different hands should be explained in the [`<handDesc>`](crate::schema::HandDesc) in the header.
    pub hand: Option<String>,
    /// The actual text of this reading
    pub text: String,
}

pub type Lacuna = crate::schema::Gap;
pub type ExtentUnit = crate::schema::ExtentUnit;
