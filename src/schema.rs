//! The schema for directly (de-)serializing into xml.

use serde::{Deserialize, Serialize};

/// Remove trailing whitespace if any exists, otherwise leave the original string untouched
fn trim_if_required(s: String) -> String {
    let trimmed = s.trim();
    if trimmed.len() == s.len() {
        s
    } else {
        trimmed.to_string()
    }
}

/// The complete TEI document.
///
/// Note that two additional lines are not included here but should be present and output to file:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <?xml-model href="file:TODO-online-schema-location" schematypens="http://relaxng.org/ns/structure/1.0" type="application/xml"?>
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Tei {
    /// MUST be `http://www.tei-c.org/ns/1.0`
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    /// The header with any meta-information
    #[serde(rename = "teiHeader")]
    pub tei_header: TeiHeader,
    /// the actual text
    pub text: Text,
}
impl Tei {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            xmlns: self.xmlns,
            tei_header: TeiHeader {
                file_desc: self.tei_header.file_desc.trim(),
            },
            text: self.text.trim(),
        }
    }
}

/// TEI Header with metainformation about a folio.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TeiHeader {
    /// TEI fileDesc element - describes this file
    #[serde(rename = "fileDesc")]
    pub file_desc: FileDesc,
}

/// TEI fileDesc element - descripbes this file.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct FileDesc {
    /// Title of this manuscript
    #[serde(rename = "titleStmt")]
    pub title_stmt: TitleStmt,
    /// How this transcription is published
    #[serde(rename = "publicationStmt")]
    pub publication_stmt: PublicationStmt,
    /// Description of the transcribed manuscript
    #[serde(rename = "sourceDesc")]
    pub source_desc: SourceDesc,
}
impl FileDesc {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            title_stmt: TitleStmt {
                title: trim_if_required(self.title_stmt.title),
            },
            publication_stmt: PublicationStmt {
                p: trim_if_required(self.publication_stmt.p),
            },
            source_desc: SourceDesc {
                ms_desc: MsDesc {
                    ms_identifier: self.source_desc.ms_desc.ms_identifier.trim(),
                    phys_desc: self.source_desc.ms_desc.phys_desc.trim(),
                },
            },
        }
    }
}

/// Title of this manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TitleStmt {
    /// Title MUST be
    /// `{manuscript-name} Folio {folio-number} {recto/verso}`
    pub title: String,
}

/// How this transcription is published.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PublicationStmt {
    /// MUST be
    /// ```xml
    /// This digital reproduction is published as part of TanakhCC and licensed as https://creativecommons.org/publicdomain/zero/1.0.
    /// ```
    pub p: String,
}

/// Description of the transcribed manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SourceDesc {
    /// TEI msDesc element
    #[serde(rename = "msDesc")]
    pub ms_desc: MsDesc,
}

/// TEI msDesc element - Description of the transcribed manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct MsDesc {
    /// Information that can identify this manuscript
    #[serde(rename = "msIdentifier")]
    pub ms_identifier: MsIdentifier,
    /// Description of the physical properties of this manuscript
    #[serde(rename = "physDesc")]
    pub phys_desc: PhysDesc,
}
impl MsDesc {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            ms_identifier: self.ms_identifier.trim(),
            phys_desc: self.phys_desc.trim(),
        }
    }
}

/// Information that can identify this manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct MsIdentifier {
    /// The institution holding this manuscript
    pub institution: Option<String>,
    /// The collection this manuscript is a part of
    pub collection: Option<String>,
    /// The name of this manuscript (NOT including folio/page numbers)
    #[serde(rename = "idno")]
    pub page_nr: String,
    #[serde(rename = "msName")]
    pub ms_name: String,
}
impl MsIdentifier {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            institution: self.institution.map(trim_if_required),
            collection: self.collection.map(trim_if_required),
            page_nr: trim_if_required(self.page_nr),
            ms_name: trim_if_required(self.ms_name),
        }
    }
}

/// Description of the physical properties of this manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PhysDesc {
    /// Description of the scribal hands present in this manuscript
    #[serde(rename = "handDesc")]
    pub hand_desc: HandDesc,
    /// Description of the scripts present in this manuscript
    #[serde(rename = "scriptDesc")]
    pub script_desc: ScriptDesc,
}
impl PhysDesc {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            hand_desc: HandDesc {
                summary: trim_if_required(self.hand_desc.summary),
            },
            script_desc: ScriptDesc {
                summary: trim_if_required(self.script_desc.summary),
            },
        }
    }
}

/// Description of the scribal hands present in this manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct HandDesc {
    /// A human readable summary of the hands present in this manuscript
    ///
    /// Different hands should be listed and given names by which they can be referred to in
    /// corrections in the [`Text`]
    pub summary: String,
}

/// Description of the scripts present in this manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ScriptDesc {
    /// A human readable summary of the scripts present in this manuscript
    ///
    /// If there are multiple languages present, a short summary should be given of the script used
    /// for each of these languages
    pub summary: String,
}

/// The entire transcribed text.
///
/// This struct is just a trivial wrapper around [`<body>`](Body), because the TEI spec requires that.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Text {
    /// the actual body
    pub body: Body,
}
impl Text {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            body: self.body.trim(),
        }
    }
}

/// The entire transcribed text body
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Body {
    /// the default language for text in this manuscript
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The columns present in this text
    #[serde(rename = "div")]
    pub columns: Vec<Column>,
}
impl Body {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            columns: self.columns.into_iter().map(Column::trim).collect(),
        }
    }
}

/// A complete column in the manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Column {
    /// The default language of text in this column
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// the type is `column`
    /// This is only enforced on the normalized type, not while (de-)serializing xml
    #[serde(rename = "@type")]
    pub div_type: String,
    /// the column number
    #[serde(rename = "@n")]
    pub n: Option<i32>,
    /// The lines in this column
    #[serde(rename = "div")]
    pub lines: Vec<Line>,
}
impl Column {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            div_type: self.div_type,
            n: self.n,
            lines: self.lines.into_iter().map(Line::trim).collect(),
        }
    }
}

/// A complete line in the manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Line {
    /// The default language of text in this line
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// the type is `line`
    /// This is only enforced on the normalized type, not while (de-)serializing xml
    #[serde(rename = "@type")]
    pub div_type: String,
    /// the line number
    #[serde(rename = "@n")]
    pub n: Option<i32>,
    /// The actual text elements contained in this line
    #[serde(rename = "$value")]
    pub blocks: Vec<InlineBlock>,
}
impl Line {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            div_type: self.div_type,
            n: self.n,
            blocks: self.blocks.into_iter().map(InlineBlock::trim).collect(),
        }
    }
}

/// A block of text or marked up text.
///
/// This maps to the Blocks (with actual content) that are editable in
/// [critic](https://github.com/curatorsigma/critic).
/// These are atomic units of text, that NEVER overlap linebreaks.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum InlineBlock {
    /// Things in a `<p>`
    ///
    /// This only exists because in the TEI spec, things like `<damage>` are in `<p>` while things like
    /// `<gap>` are not in a `<p>`
    #[serde(rename = "p")]
    P(TDOCWrapper),
    /// A lacuna in the manuscript
    #[serde(rename = "gap")]
    Gap(Gap),
    /// An anchor - the beginning of a verse
    #[serde(rename = "anchor")]
    Anchor(Anchor),
    /// A correction in the manuscript - where one scribal hand has overwritte / struck through / .. a text that was present earlier
    #[serde(rename = "app")]
    App(App),
    /// A bit of significant space
    #[serde(rename = "space")]
    Space(Space),
}
impl InlineBlock {
    /// Trim whitespace from all text fields
    #[must_use]
    pub fn trim(self) -> Self {
        match self {
            Self::Space(_) | Self::Gap(_) | Self::Anchor(_) => self,
            Self::P(x) => Self::P(x.trim()),
            Self::App(x) => Self::App(x.trim()),
        }
    }
}

/// Intermediate Wrapper struct required for XML (de-)serialization.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TDOCWrapper {
    /// The language set on the `<p>` element
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The actual content we care about
    #[serde(rename = "$value")]
    pub value: TextDamageOrChoice,
}
impl TDOCWrapper {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            value: self.value.trim(),
        }
    }
}

/// Anything that can be in a p in normal flowing text.
///
/// This is a bit of a weird support-type, since it is necessary for writing out this schema but
/// has no actual semantic.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TextDamageOrChoice {
    /// Raw text - this is text clearly visible in the manuscript
    #[serde(rename = "$text")]
    Text(String),
    /// Damaged text
    #[serde(rename = "damage")]
    Damage(Damage),
    /// An expanded abbreviation
    #[serde(rename = "choice")]
    Choice(Choice),
}
impl TextDamageOrChoice {
    #[must_use]
    pub fn trim(self) -> Self {
        match self {
            Self::Text(x) => Self::Text(trim_if_required(x)),
            Self::Damage(x) => Self::Damage(x.trim()),
            Self::Choice(x) => Self::Choice(x.trim()),
        }
    }
}

/// The beginning of a verse.
///
/// This could either be marked in the manuscript (Sof Passuq, verse number etc.) or supplied from
/// other manuscripts if it is certain which verse begins at this point.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Anchor {
    /// The ID of this verse.
    ///
    /// MUST be `A_V_{versification-theme-shorthand}_{verse-number}`
    #[serde(rename = "@xml:id")]
    pub xml_id: String,
    /// MUST be `{versification-theme-long-form}`
    #[serde(rename = "@type")]
    pub anchor_type: String,
}

/// A damaged part of the text that is still legible, but hard to read.
///
/// If a part of the text is illegible, use [`<gap>`](Gap) instead.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Damage {
    /// The language set on the `<damage>` element
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The certainty the transcriber assigns to the reconstruction of the damaged text
    #[serde(rename = "@cert", skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,
    /// The cause of damage
    #[serde(rename = "@agent")]
    pub agent: String,
    /// The reproduction of the damaged text
    #[serde(rename = "$text")]
    pub content: String,
}
impl Damage {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            cert: self.cert,
            agent: self.agent,
            content: trim_if_required(self.content),
        }
    }
}

/// An expanded Abbreviation.
///
/// For corrections, use [`<app>`](App) instead.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Choice {
    /// The language set on the `<choice>` element
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The surface form (the abbreviation) present in the manuscript
    #[serde(rename = "abbr")]
    pub surface: AbbrSurface,
    /// The expanded form supplied by the transcriber
    #[serde(rename = "expan")]
    pub expansion: AbbrExpansion,
}
impl Choice {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            surface: self.surface.trim(),
            expansion: self.expansion.trim(),
        }
    }
}

/// The surface form of the abbreviation
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AbbrSurface {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub content: String,
}
impl AbbrSurface {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            content: trim_if_required(self.content),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AbbrExpansion {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "$text")]
    pub content: String,
}
impl AbbrExpansion {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            content: trim_if_required(self.content),
        }
    }
}

/// An ancient correction.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct App {
    /// The language set on the `<app>` element
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// A list of different readings. Each form this manuscript had at one point should get its own
    /// reading and be written out in its entirety here.
    pub rdg: Vec<Rdg>,
}
impl App {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            rdg: self.rdg.into_iter().map(Rdg::trim).collect(),
        }
    }
}

/// An individual reading (version) inside a correction.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Rdg {
    /// the language set on the `<rdg>`
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The scribal hand responsible for this reading
    ///
    /// The different hands should be explained in the [`HandDesc`] in the header.
    #[serde(rename = "@hand", skip_serializing_if = "Option::is_none")]
    pub hand: Option<String>,
    /// The number of this reading (i.e. 1 for the first version, 2 for the second version etc.)
    #[serde(rename = "@varSeq")]
    pub var_seq: i32,
    /// The actual text of this reading
    #[serde(rename = "$text")]
    pub content: String,
}
impl Rdg {
    #[must_use]
    pub fn trim(self) -> Self {
        Self {
            lang: self.lang,
            hand: self.hand,
            var_seq: self.var_seq,
            content: trim_if_required(self.content),
        }
    }
}

/// A lacuna.
///
/// For damaged but legible text, use [`<damage>`](Damage) instead.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Gap {
    /// The reason this text is lacunous
    #[serde(rename = "@reason")]
    pub reason: String,
    /// The unit in which the approximate extent of this lacuna is given
    #[serde(rename = "@unit")]
    pub unit: ExtentUnit,
    /// The extent of this lacuna in the given unit
    #[serde(rename = "@n")]
    pub n: i32,
    /// The certainty for the approximate extent AND the proposed content
    ///
    /// If not content is proposed, the certainty for the approximate extent
    #[serde(rename = "@cert", skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,
}
impl Default for Gap {
    fn default() -> Self {
        Self {
            unit: ExtentUnit::default(),
            n: 1,
            reason: String::default(),
            cert: None,
        }
    }
}

/// A bit of significant space in the manuscript
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Space {
    /// The size of this whitespace in multiples of the given [`unit`](Self::unit)
    #[serde(rename = "@quantity")]
    pub quantity: i32,
    /// The unit for this whitespace.
    #[serde(rename = "@unit")]
    pub unit: ExtentUnit,
}
impl Default for Space {
    fn default() -> Self {
        Self {
            quantity: 2,
            unit: ExtentUnit::Character,
        }
    }
}

/// The unit used to express extent of a part of Text.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExtentUnit {
    /// Single character
    #[serde(rename = "character")]
    Character,
    /// A line
    #[serde(rename = "line")]
    Line,
    /// A column
    #[serde(rename = "column")]
    Column,
}
/// Default for user facing code
impl Default for ExtentUnit {
    fn default() -> Self {
        Self::Character
    }
}
impl core::str::FromStr for ExtentUnit {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Character" => Ok(Self::Character),
            "Line" => Ok(Self::Line),
            "Column" => Ok(Self::Column),
            _ => Err(()),
        }
    }
}
impl ExtentUnit {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Character => "Character",
            Self::Line => "Line",
            Self::Column => "Column",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extent_name_roundtrip() {
        let x = ExtentUnit::Character;
        assert_eq!(x, x.name().parse().unwrap());
        let x = ExtentUnit::Line;
        assert_eq!(x, x.name().parse().unwrap());
        let x = ExtentUnit::Column;
        assert_eq!(x, x.name().parse().unwrap());
    }

    #[test]
    fn choice() {
        let xml = r#"<choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                lang: None,
                surface: AbbrSurface {
                    lang: None,
                    content: "JHWH".to_string(),
                },
                expansion: AbbrExpansion {
                    lang: None,
                    content: "Jahwe".to_string(),
                },
            }
        );
    }

    /// adding superfluous elements is irrelevant - this is not always tested
    #[test]
    fn choice_added_text() {
        let xml = r#"<choice>text<abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                lang: None,
                surface: AbbrSurface {
                    lang: None,
                    content: "JHWH".to_string(),
                },
                expansion: AbbrExpansion {
                    lang: None,
                    content: "Jahwe".to_string(),
                },
            }
        );
    }

    /// changing order is also irrelevant - not always tested
    #[test]
    fn choice_changed_order() {
        let xml = r#"<choice><expan>Jahwe</expan><abbr>JHWH</abbr></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                lang: None,
                surface: AbbrSurface {
                    lang: None,
                    content: "JHWH".to_string(),
                },
                expansion: AbbrExpansion {
                    lang: None,
                    content: "Jahwe".to_string(),
                },
            }
        );
    }

    /// Missing elements lead to errors - not always tested
    #[test]
    fn choice_missing_expan() {
        let xml = r#"<choice><abbr>JHWH</abbr></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// base case - everything is here
    #[test]
    fn gap() {
        let xml = r#"<gap reason="lost" n="2" unit="column" cert="high"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Column,
                cert: Some("high".to_string()),
            }
        );
    }

    /// cert is optional
    #[test]
    fn gap_no_cert() {
        let xml = r#"<gap reason="lost" n="2" unit="line"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Line,
                cert: None,
            }
        );
    }

    /// only line character and column are supported
    #[test]
    fn gap_allowed_units_correct() {
        let xml = r#"<gap reason="lost" n="2" unit="character"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Character,
                cert: None,
            }
        );

        let xml = r#"<gap reason="lost" n="2" unit="line"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Line,
                cert: None,
            }
        );

        let xml = r#"<gap reason="lost" n="2" unit="column"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Column,
                cert: None,
            }
        );

        let xml = r#"<gap reason="lost" n="2" unit="does_not_exist"/>"#;
        let result: Result<Gap, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// rdg - base case
    #[test]
    fn rdg() {
        let xml = r#"<rdg hand="handname" varSeq="1">The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Rdg {
                lang: None,
                hand: Some("handname".to_string()),
                var_seq: 1,
                content: "The content.".to_string(),
            }
        );
    }

    /// rdg - hand is optional
    #[test]
    fn rdg_no_hand() {
        let xml = r#"<rdg varSeq="1">The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Rdg {
                lang: None,
                hand: None,
                var_seq: 1,
                content: "The content.".to_string(),
            }
        );
    }

    /// rdg - varSeq is not optional
    #[test]
    fn rdg_no_varseq() {
        let xml = r#"<rdg>The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// rdg - text is not optional
    #[test]
    fn rdg_no_text() {
        let xml = r#"<rdg varSeq="3"/>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// app - a list of rdgs
    #[test]
    fn app() {
        let xml = r#"<app><rdg varSeq="1">Content1</rdg><rdg varSeq="2">Content2</rdg></app>"#;
        let result: Result<App, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            App {
                lang: None,
                rdg: vec![
                    Rdg {
                        lang: None,
                        hand: None,
                        var_seq: 1,
                        content: "Content1".to_string()
                    },
                    Rdg {
                        lang: None,
                        hand: None,
                        var_seq: 2,
                        content: "Content2".to_string()
                    }
                ]
            }
        );
    }

    /// damage - base case
    #[test]
    fn damage() {
        let xml = r#"<damage cert="low" agent="water">damaged</damage>"#;
        let result: Result<Damage, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Damage {
                lang: None,
                cert: Some("low".to_string()),
                agent: "water".to_string(),
                content: "damaged".to_string()
            }
        );
    }

    /// damage - with language
    #[test]
    fn damage_with_lang() {
        let xml = r#"<damage xml:lang="en" cert="low" agent="water">damaged</damage>"#;
        let result: Result<Damage, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Damage {
                lang: Some("en".to_string()),
                cert: Some("low".to_string()),
                agent: "water".to_string(),
                content: "damaged".to_string()
            }
        );
    }

    /// anchor - base case
    #[test]
    fn anchor() {
        let xml = r#"<anchor xml:id="A_V_MT_1Kg-3-4" type="Masoretic"/>"#;
        let result: Result<Anchor, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Anchor {
                xml_id: "A_V_MT_1Kg-3-4".to_string(),
                anchor_type: "Masoretic".to_string(),
            }
        );
    }

    /// TextDamageOrChoice - text
    #[test]
    fn text_damage_or_choice_text() {
        let xml = r#"raw stuff"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TextDamageOrChoice::Text("raw stuff".to_string())
        );
    }

    /// TextDamageOrChoice - damage
    #[test]
    fn text_damage_or_choice_damage() {
        let xml = r#"<damage cert="low" agent="water">damaged</damage>"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TextDamageOrChoice::Damage(Damage {
                lang: None,
                cert: Some("low".to_string()),
                agent: "water".to_string(),
                content: "damaged".to_string()
            })
        );
    }

    /// TextDamageOrChoice - Choice
    #[test]
    fn text_damage_or_choice_choice() {
        let xml = r#"<choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TextDamageOrChoice::Choice(Choice {
                lang: None,
                surface: AbbrSurface {
                    lang: None,
                    content: "JHWH".to_string(),
                },
                expansion: AbbrExpansion {
                    lang: None,
                    content: "Jahwe".to_string(),
                },
            })
        );
    }

    /// TextDamageOrChoice - Choice
    #[test]
    fn text_damage_or_choice_other() {
        let xml = r#"<app/>"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// InlineBlock - P - text
    #[test]
    fn inline_block_p() {
        let xml = r#"<p>text</p>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        dbg!(&result);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::P(TDOCWrapper {
                lang: None,
                value: TextDamageOrChoice::Text("text".to_string())
            })
        );
    }

    /// InlineBlock - p - damaged
    #[test]
    fn inline_block_p_damaged() {
        let xml = r#"<p><damage cert="low" agent="water">damaged</damage></p>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        dbg!(&result);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::P(TDOCWrapper {
                lang: None,
                value: TextDamageOrChoice::Damage(Damage {
                    lang: None,
                    cert: Some("low".to_string()),
                    agent: "water".to_string(),
                    content: "damaged".to_string()
                })
            })
        );
    }

    /// InlineBlock - p - choice
    #[test]
    fn inline_block_p_choice() {
        let xml = r#"<p><choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice></p>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::P(TDOCWrapper {
                lang: None,
                value: TextDamageOrChoice::Choice(Choice {
                    lang: None,
                    surface: AbbrSurface {
                        lang: None,
                        content: "JHWH".to_string(),
                    },
                    expansion: AbbrExpansion {
                        lang: None,
                        content: "Jahwe".to_string(),
                    },
                })
            })
        );
    }

    /// InlineBlock - p - choice
    #[test]
    fn inline_block_p_w_lang() {
        let xml = r#"<p xml:lang="hbo-Hebr-x-babli"><choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice></p>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::P(TDOCWrapper {
                lang: Some("hbo-Hebr-x-babli".to_string()),
                value: TextDamageOrChoice::Choice(Choice {
                    lang: None,
                    surface: AbbrSurface {
                        lang: None,
                        content: "JHWH".to_string(),
                    },
                    expansion: AbbrExpansion {
                        lang: None,
                        content: "Jahwe".to_string(),
                    },
                })
            })
        );
    }

    /// InlineBlock - Gap
    #[test]
    fn inline_block_gap() {
        let xml = r#"<gap reason="lost" n="2" unit="column"/>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::Gap(Gap {
                reason: "lost".to_string(),
                n: 2,
                unit: ExtentUnit::Column,
                cert: None,
            })
        );
    }

    /// InlineBlock - Anchor
    #[test]
    fn inline_block_anchor() {
        let xml = r#"<anchor xml:id="A_V_MT_1Kg-3-4" type="Masoretic"/>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::Anchor(Anchor {
                xml_id: "A_V_MT_1Kg-3-4".to_string(),
                anchor_type: "Masoretic".to_string(),
            })
        );
    }

    /// InlineBlock - App
    #[test]
    fn inline_block_app() {
        let xml = r#"<app><rdg varSeq="1">Content1</rdg><rdg varSeq="2">Content2</rdg></app>"#;
        let result: Result<InlineBlock, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            InlineBlock::App(App {
                lang: None,
                rdg: vec![
                    Rdg {
                        lang: None,
                        hand: None,
                        var_seq: 1,
                        content: "Content1".to_string()
                    },
                    Rdg {
                        lang: None,
                        hand: None,
                        var_seq: 2,
                        content: "Content2".to_string()
                    },
                ]
            })
        );
    }

    /// Line - base case
    #[test]
    fn line() {
        let xml =
            r#"<div type="line" n="3"><anchor xml:id="A_V_MT_1Kg-3-4" type="Masoretic"/></div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Line {
                lang: None,
                div_type: "line".to_string(),
                n: Some(3),
                blocks: vec![InlineBlock::Anchor(Anchor {
                    xml_id: "A_V_MT_1Kg-3-4".to_string(),
                    anchor_type: "Masoretic".to_string(),
                })]
            }
        );
    }

    /// Line - with a bit of whitespace added
    #[test]
    fn line_w_whitespace() {
        let xml = r#"<div xml:lang="grc" type="line" n="3">
                <anchor xml:id="A_V_LXX_1Kg-3-4" type="Septuagint"/>
            </div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Line {
                lang: Some("grc".to_string()),
                div_type: "line".to_string(),
                n: Some(3),
                blocks: vec![InlineBlock::Anchor(Anchor {
                    xml_id: "A_V_LXX_1Kg-3-4".to_string(),
                    anchor_type: "Septuagint".to_string(),
                })]
            }
        );
    }

    /// Line - direct text not allowed
    #[test]
    fn line_direct_text() {
        let xml = r#"<div type="line" n="3">text here must be in p</div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// Line - type must be given
    #[test]
    fn line_no_type() {
        let xml = r#"<div n="3"><p>text is allowed because it is in a </p></div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// Line - at least one block must exist
    #[test]
    fn line_no_block() {
        let xml = r#"<div n="3"></div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// Line - several blocks and no n
    #[test]
    fn line_multiblock_no_n() {
        let xml = r#"<div type="line"><gap reason="lost" n="2" unit="column"/><anchor xml:id="A_V_MT_1Kg-3-4" type="Masoretic"/><p><damage cert="low" agent="water">damaged</damage></p></div>"#;
        let result: Result<Line, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Line {
                lang: None,
                div_type: "line".to_string(),
                n: None,
                blocks: vec![
                    InlineBlock::Gap(Gap {
                        reason: "lost".to_string(),
                        n: 2,
                        unit: ExtentUnit::Column,
                        cert: None,
                    }),
                    InlineBlock::Anchor(Anchor {
                        xml_id: "A_V_MT_1Kg-3-4".to_string(),
                        anchor_type: "Masoretic".to_string(),
                    }),
                    InlineBlock::P(TDOCWrapper {
                        lang: None,
                        value: TextDamageOrChoice::Damage(Damage {
                            lang: None,
                            cert: Some("low".to_string()),
                            agent: "water".to_string(),
                            content: "damaged".to_string()
                        })
                    })
                ]
            }
        );
    }

    /// Column with two lines and differing languages
    #[test]
    fn column_multiline_multiblock() {
        let xml = r#"<div type="column" n="1" xml:lang="hbo-Hebr-x-babli">
            <div type="line" xml:lang="hbo-Hebr">
                <gap reason="lost" n="2" unit="column"/>
                <anchor xml:id="A_V_MT_1Kg-3-4" type="Masoretic"/>
                <p><damage cert="low" agent="water">damaged</damage></p>
            </div>
            <div type="line" n="3">
                <anchor xml:id="A_V_MT_1Kg-3-5" type="Masoretic"/>
            </div>
            </div>"#;
        let result: Result<Column, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Column {
                lang: Some("hbo-Hebr-x-babli".to_string()),
                n: Some(1),
                div_type: "column".to_string(),
                lines: vec![
                    Line {
                        lang: Some("hbo-Hebr".to_string()),
                        div_type: "line".to_string(),
                        n: None,
                        blocks: vec![
                            InlineBlock::Gap(Gap {
                                reason: "lost".to_string(),
                                n: 2,
                                unit: ExtentUnit::Column,
                                cert: None,
                            }),
                            InlineBlock::Anchor(Anchor {
                                xml_id: "A_V_MT_1Kg-3-4".to_string(),
                                anchor_type: "Masoretic".to_string(),
                            }),
                            InlineBlock::P(TDOCWrapper {
                                lang: None,
                                value: TextDamageOrChoice::Damage(Damage {
                                    lang: None,
                                    cert: Some("low".to_string()),
                                    agent: "water".to_string(),
                                    content: "damaged".to_string()
                                })
                            })
                        ]
                    },
                    Line {
                        lang: None,
                        div_type: "line".to_string(),
                        n: Some(3),
                        blocks: vec![InlineBlock::Anchor(Anchor {
                            xml_id: "A_V_MT_1Kg-3-5".to_string(),
                            anchor_type: "Masoretic".to_string(),
                        })]
                    }
                ]
            }
        );
    }

    /// Column without content is not allowed - at least one line must exist
    #[test]
    fn column_no_lines() {
        let xml = r#"<div type="column" n="3"></div>"#;
        let result: Result<Column, _> = quick_xml::de::from_str(xml);
        dbg!(&result);
        assert!(result.is_err());
    }

    /// Column with one line is allowed
    #[test]
    fn column_single_line() {
        let xml = r#"<div type="column" n="1" xml:lang="hbo-Hebr-x-babli">
            <div type="line" n="3">
                <anchor xml:id="A_V_MT_1Kg-3-5" type="Masoretic"/>
            </div>
            </div>"#;
        let result: Result<Column, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Column {
                lang: Some("hbo-Hebr-x-babli".to_string()),
                n: Some(1),
                div_type: "column".to_string(),
                lines: vec![Line {
                    lang: None,
                    div_type: "line".to_string(),
                    n: Some(3),
                    blocks: vec![InlineBlock::Anchor(Anchor {
                        xml_id: "A_V_MT_1Kg-3-5".to_string(),
                        anchor_type: "Masoretic".to_string(),
                    })]
                }]
            }
        );
    }

    /// Body with a single column
    #[test]
    fn body_1_by_1() {
        let xml = r#"<body xml:lang="grc">
            <div type="column" n="1" xml:lang="hbo-Hebr-x-babli">
            <div type="line" n="3">
                <anchor xml:id="A_V_MT_1Kg-3-5" type="Masoretic"/>
            </div>
            </div>
            </body>"#;
        let result: Result<Body, _> = quick_xml::de::from_str(xml);
        dbg!(&result);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Body {
                lang: Some("grc".to_string()),
                columns: vec![Column {
                    lang: Some("hbo-Hebr-x-babli".to_string()),
                    n: Some(1),
                    div_type: "column".to_string(),
                    lines: vec![Line {
                        lang: None,
                        div_type: "line".to_string(),
                        n: Some(3),
                        blocks: vec![InlineBlock::Anchor(Anchor {
                            xml_id: "A_V_MT_1Kg-3-5".to_string(),
                            anchor_type: "Masoretic".to_string(),
                        })]
                    }]
                }]
            }
        );
    }

    /// Body - no language is allowed. Setting a master language will only be enforced when
    /// normalizing
    #[test]
    fn body_2_by_1() {
        let xml = r#"<body>
            <div type="column" n="1" xml:lang="hbo-Hebr-x-babli">
            <div type="line" n="3">
                <anchor xml:id="A_V_MT_1Kg-3-5" type="Masoretic"/>
            </div>
            </div>
            <div type="column" n="2" xml:lang="hbo-Hebr">
            <div type="line" n="1">
                <p>
                    Some text here
                </p>
            </div>
            </div>
            </body>"#;
        let result: Result<Body, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().trim(),
            Body {
                lang: None,
                columns: vec![
                    Column {
                        lang: Some("hbo-Hebr-x-babli".to_string()),
                        n: Some(1),
                        div_type: "column".to_string(),
                        lines: vec![Line {
                            lang: None,
                            div_type: "line".to_string(),
                            n: Some(3),
                            blocks: vec![InlineBlock::Anchor(Anchor {
                                xml_id: "A_V_MT_1Kg-3-5".to_string(),
                                anchor_type: "Masoretic".to_string(),
                            })]
                        }]
                    },
                    Column {
                        lang: Some("hbo-Hebr".to_string()),
                        n: Some(2),
                        div_type: "column".to_string(),
                        lines: vec![Line {
                            lang: None,
                            div_type: "line".to_string(),
                            n: Some(1),
                            blocks: vec![InlineBlock::P(TDOCWrapper {
                                lang: None,
                                value: TextDamageOrChoice::Text("Some text here".to_string())
                            })]
                        }]
                    }
                ]
            }
        );
    }

    /// Text - simply wraps a body
    #[test]
    fn text() {
        let xml = r#"
            <text>
            <body xml:lang="grc">
            <div type="column" n="1" xml:lang="hbo-Hebr-x-babli">
            <div type="line" n="3">
                <anchor xml:id="A_V_MT_1Kg-3-5" type="Masoretic"/>
            </div>
            </div>
            <div type="column" n="2" xml:lang="hbo-Hebr">
            <div type="line" n="1">
                <p>
                    Some text here
                </p>
            </div>
            </div>
            </body>
            </text>"#;
        let result: Result<Text, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().trim(),
            Text {
                body: Body {
                    lang: Some("grc".to_string()),
                    columns: vec![
                        Column {
                            lang: Some("hbo-Hebr-x-babli".to_string()),
                            n: Some(1),
                            div_type: "column".to_string(),
                            lines: vec![Line {
                                lang: None,
                                div_type: "line".to_string(),
                                n: Some(3),
                                blocks: vec![InlineBlock::Anchor(Anchor {
                                    xml_id: "A_V_MT_1Kg-3-5".to_string(),
                                    anchor_type: "Masoretic".to_string(),
                                })]
                            }]
                        },
                        Column {
                            lang: Some("hbo-Hebr".to_string()),
                            n: Some(2),
                            div_type: "column".to_string(),
                            lines: vec![Line {
                                lang: None,
                                div_type: "line".to_string(),
                                n: Some(1),
                                blocks: vec![InlineBlock::P(TDOCWrapper {
                                    lang: None,
                                    value: TextDamageOrChoice::Text("Some text here".to_string())
                                })]
                            }]
                        }
                    ]
                }
            }
        );
    }

    /// an entire manuscript
    #[test]
    fn tei() {
        let xml = include_str!("../examples/01_all_elements.xml");
        let result: Result<Tei, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());

        let tei = Tei {
        xmlns: "http://www.tei-c.org/ns/1.0".to_string(),
        tei_header: TeiHeader {
            file_desc: FileDesc {
                title_stmt: TitleStmt {
                    title: "Manuskript Name folio 34 verso.".to_string(),
                },
                publication_stmt: PublicationStmt {
                    p: "This digital reproduction is published as part of TanakhCC and licensed as https://creativecommons.org/publicdomain/zero/1.0.".to_string(),
                },
                source_desc: SourceDesc {
                    ms_desc: MsDesc {
                        ms_identifier: MsIdentifier {
                            institution: Some(
                                "University of does-not-exist".to_string(),
                            ),
                            collection: Some(
                                "Collectors Edition 2 electric boogaloo".to_string(),
                            ),
                            ms_name: "Der Name voms dem Manuskripts".to_string(),
                            page_nr: "34 verso".to_string(),
                        },
                        phys_desc: PhysDesc {
                            hand_desc: HandDesc {
                                summary: "There are two recognizable Hands: hand1 and hand2.".to_string(),
                            },
                            script_desc: ScriptDesc {
                                summary: "Die Schrift in diesem Manuskript gibt es.".to_string(),
                            },
                        },
                    },
                },
            },
        },
        text: Text {
            body: Body {
                lang: Some(
                    "hbo-Hebr".to_string(),
                ),
                columns: vec![
                    Column {
                        lang: None,
                        div_type: "column".to_string(),
                        n: Some(
                            1,
                        ),
                        lines: vec![
                            Line {
                                lang: None,
                                div_type: "line".to_string(),
                                n: Some(
                                    2,
                                ),
                                blocks: vec![
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Text(
                                                "asdfa".to_string(),
                                            ),
                                        },
                                    ),
                                    InlineBlock::Anchor(
                                        Anchor {
                                            xml_id: "A_V_MT_1Kg-3-4".to_string(),
                                            anchor_type: "Masoretic".to_string(),
                                        },
                                    ),
                                    InlineBlock::Anchor(
                                        Anchor {
                                            xml_id: "A_V_LXX_1Kg-3-4".to_string(),
                                            anchor_type: "Septuagint".to_string(),
                                        },
                                    ),
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Text(
                                                "sdfsa".to_string(),
                                            ),
                                        },
                                    ),
                                ],
                            },
                            Line {
                                lang: Some(
                                    "hbo-Hebr-x-babli".to_string(),
                                ),
                                div_type: "line".to_string(),
                                n: None,
                                blocks: vec![
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Text(
                                                "Some stuff with babylonian Niqud".to_string(),
                                            ),
                                        },
                                    ),
                                ],
                            },
                        ],
                    },
                    Column {
                        lang: None,
                        div_type: "column".to_string(),
                        n: None,
                        lines: vec![
                            Line {
                                lang: None,
                                div_type: "line".to_string(),
                                n: Some(
                                    2,
                                ),
                                blocks: vec![
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Text(
                                                "Hier ein an".to_string(),
                                            ),
                                        },
                                    ),
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Damage(
                                                Damage {
                                                    lang: None,
                                                    cert: Some("high".to_string()),
                                                    agent: "water".to_string(),
                                                    content: "d".to_string(),
                                                },
                                            ),
                                        },
                                    ),
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Text(
                                                "erer, wo der Buchstabe nur etwas kaputt ist.".to_string(),
                                            ),
                                        },
                                    ),
                                    InlineBlock::Gap(
                                        Gap {
                                            reason: "lost".to_string(),
                                            n: 12,
                                            unit: ExtentUnit::Character,
                                            cert: Some(
                                                "0.10".to_string(),
                                            ),
                                        },
                                    ),
                                    InlineBlock::P(
                                        TDOCWrapper {
                                            lang: None,
                                            value: TextDamageOrChoice::Choice(
                                                Choice {
                                                    lang: None,
                                                    surface: AbbrSurface {
                                                        lang: None,
                                                        content: "JHWH".to_string(),
                                                    },
                                                    expansion: AbbrExpansion {
                                                        lang: None,
                                                        content: "Jahwe".to_string(),
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    InlineBlock::App(
                                        App {
                                            lang: None,
                                            rdg: vec![
                                                Rdg {
                                                    lang: None,
                                                    hand: Some(
                                                        "hand1".to_string(),
                                                    ),
                                                    var_seq: 1,
                                                    content: "sam stuff 1".to_string(),
                                                },
                                                Rdg {
                                                    lang: None,
                                                    hand: Some(
                                                        "hand2".to_string(),
                                                    ),
                                                    var_seq: 2,
                                                    content: "sam stuff 2".to_string(),
                                                },
                                            ],
                                        },
                                    ),
                                ],
                            },
                        ],
                    },
                ],
            },
        },
    };
        let deserialized_trimmed = result.unwrap().trim();
        assert_eq!(deserialized_trimmed, tei);
    }

    // https://github.com/tafia/quick-xml/issues/841
    #[test]
    fn lang_attribute_serialized_with_xml() {
        let dmg = Damage {
            lang: Some("language".to_string()),
            cert: Some("high".to_string()),
            agent: "agent".to_string(),
            content: "text".to_string(),
        };
        let sr = quick_xml::se::to_string(&dmg);
        dbg!(&sr);
        assert_eq!(
            sr.unwrap(),
            r#"<Damage xml:lang="language" cert="high" agent="agent">text</Damage>"#.to_string()
        );
    }

    /// None Language should roundtrip to None
    #[test]
    fn none_language_ser_deser() {
        let block = Damage {
            lang: None,
            cert: Some("high".to_string()),
            agent: "agent".to_string(),
            content: "text".to_string(),
        };
        let sr = quick_xml::se::to_string(&block).unwrap();
        let ds: Damage = quick_xml::de::from_str(&sr).unwrap();
        assert_eq!(ds, block);
    }

    /// None cert should roundtrip to None
    #[test]
    fn none_cert_ser_deser() {
        let block = Gap {
            reason: "reason".to_string(),
            unit: ExtentUnit::Line,
            n: 1,
            cert: None,
        };
        let sr = quick_xml::se::to_string(&block).unwrap();
        let ds: Gap = quick_xml::de::from_str(&sr).unwrap();
        assert_eq!(ds, block);
    }

    /// None cert should roundtrip to None
    #[test]
    fn none_hand_ser_deser() {
        let block = Rdg {
            lang: None,
            hand: None,
            var_seq: 1,
            content: "text".to_string(),
        };
        let sr = quick_xml::se::to_string(&block).unwrap();
        let ds: Rdg = quick_xml::de::from_str(&sr).unwrap();
        assert_eq!(ds, block);
    }

    #[test]
    fn gap_with_content() {
        let xml = r#"<gap reason="lost" unit="column" n="2" cert="high"/>"#;
        let expected = Gap {
            reason: "lost".to_string(),
            n: 2,
            unit: ExtentUnit::Column,
            cert: Some("high".to_string()),
        };
        let deser: Gap = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(deser, expected);
        let sr = quick_xml::se::to_string_with_root("gap", &expected).unwrap();
        assert_eq!(sr, xml);
    }

    #[test]
    fn gap_without_cert() {
        let xml = r#"<gap reason="lost" unit="column" n="2"/>"#;
        let expected = Gap {
            reason: "lost".to_string(),
            n: 2,
            unit: ExtentUnit::Column,
            cert: None,
        };
        let deser: Gap = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(deser, expected);
        let sr = quick_xml::se::to_string_with_root("gap", &expected).unwrap();
        assert_eq!(sr, xml);
    }

    #[test]
    fn damage_without_cert() {
        let xml = r#"<damage agent="water">content</damage>"#;
        let expected = Damage {
            lang: None,
            cert: None,
            agent: "water".to_string(),
            content: "content".to_string(),
        };
        let deser: Damage = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(expected, deser);
        let ser = quick_xml::se::to_string_with_root("damage", &deser).unwrap();
        assert_eq!(ser, xml);
    }

    #[test]
    fn choice_with_complex_languages() {
        let xml = r#"<choice xml:lang="IRRELEVANT"><abbr xml:lang="grc">πιπι</abbr><expan xml:lang="hbo-Hebr">יהוה</expan></choice>"#;
        let expected = Choice {
            lang: Some("IRRELEVANT".to_string()),
            surface: AbbrSurface {
                lang: Some("grc".to_string()),
                content: "πιπι".to_string(),
            },
            expansion: AbbrExpansion {
                lang: Some("hbo-Hebr".to_string()),
                content: "יהוה".to_string(),
            },
        };
        let deser: Choice = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(expected, deser);
        let ser = quick_xml::se::to_string_with_root("choice", &deser).unwrap();
        assert_eq!(ser, xml);

        let xml = r#"<choice><abbr xml:lang="grc">πιπι</abbr><expan xml:lang="hbo-Hebr">יהוה</expan></choice>"#;
        let expected = Choice {
            lang: None,
            surface: AbbrSurface {
                lang: Some("grc".to_string()),
                content: "πιπι".to_string(),
            },
            expansion: AbbrExpansion {
                lang: Some("hbo-Hebr".to_string()),
                content: "יהוה".to_string(),
            },
        };
        let deser: Choice = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(expected, deser);
        let ser = quick_xml::se::to_string_with_root("choice", &deser).unwrap();
        assert_eq!(ser, xml);
    }

    #[test]
    fn space() {
        let xml = r#"<space quantity="7" unit="character"/>"#;
        let expected = Space {
            quantity: 7,
            unit: ExtentUnit::Character,
        };
        let deser: Space = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(expected, deser);
        let ser = quick_xml::se::to_string_with_root("space", &deser).unwrap();
        assert_eq!(ser, xml);
    }
}
