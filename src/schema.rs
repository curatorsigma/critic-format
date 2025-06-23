use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tei {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "$text")]
    pub text_content: Option<String>,
    #[serde(rename = "teiHeader")]
    pub tei_header: TeiHeader,
    pub text: Text,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeiHeader {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "fileDesc")]
    pub file_desc: FileDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "titleStmt")]
    pub title_stmt: TitleStmt,
    #[serde(rename = "publicationStmt")]
    pub publication_stmt: PublicationStmt,
    #[serde(rename = "sourceDesc")]
    pub source_desc: SourceDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TitleStmt {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicationStmt {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub p: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "msDesc")]
    pub ms_desc: MsDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "msIdentifier")]
    pub ms_identifier: MsIdentifier,
    #[serde(rename = "physDesc")]
    pub phys_desc: PhysDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsIdentifier {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub institution: String,
    pub collection: String,
    #[serde(rename = "msName")]
    pub ms_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhysDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "handDesc")]
    pub hand_desc: HandDesc,
    #[serde(rename = "scriptDesc")]
    pub script_desc: ScriptDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HandDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptDesc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Text {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body {
    #[serde(rename = "@lang")]
    pub xml_lang: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub div: Vec<Column>,
}

/// A complete column in the manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Column {
    /// The default language of text in this column
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
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

/// A complete line in the manuscript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Line {
    /// The default language of text in this line
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
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

/// A block of text or marked up text.
///
/// This maps to the Blocks (with actual content) that are editable in
/// [critic](https://github.com/curatorsigma/critic).
/// These are atomic units of text, that NEVER overlap linebreaks.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InlineBlock {
    #[serde(rename = "p")]
    P(TDOCWrapper),
    #[serde(rename = "gap")]
    Gap(Gap),
    #[serde(rename = "anchor")]
    Anchor(Anchor),
}

/// Intermediate Wrapper struct required for XML (de-)serialization.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TDOCWrapper {
    #[serde(rename = "@lang")]
    xml_lang: Option<String>,
    #[serde(rename = "$value")]
    value: TextDamageOrChoice,
}

/// Anything that can be in a p in normal flowing text.
///
/// This is a bit of a weird support-type, since it is necessary for writing out this schema but
/// has no actual semantic.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum TextDamageOrChoice {
    #[serde(rename = "$text")]
    Text(String),
    #[serde(rename = "damage")]
    Damage(Damage),
    #[serde(rename = "choice")]
    Choice(Choice),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Anchor {
    #[serde(rename = "@id")]
    pub xml_id: String,
    #[serde(rename = "@type")]
    pub anchor_type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Damage {
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
    #[serde(rename = "@cert")]
    pub cert: String,
    #[serde(rename = "@agent")]
    pub agent: String,
    #[serde(rename = "$text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Choice {
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
    pub abbr: String,
    pub expan: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct App {
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
    pub rdg: Vec<Rdg>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Rdg {
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
    #[serde(rename = "@hand")]
    pub hand: Option<String>,
    #[serde(rename = "@varSeq")]
    pub var_seq: i32,
    #[serde(rename = "$text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Gap {
    #[serde(rename = "@reason")]
    pub reason: String,
    #[serde(rename = "@n")]
    pub n: i32,
    #[serde(rename = "@unit")]
    pub unit: ExtentUnit,
    #[serde(rename = "@cert")]
    pub cert: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ExtentUnit {
    #[serde(rename = "character")]
    Character,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "column")]
    Column,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deser_choice() {
        let xml = r#"<choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                xml_lang: None,
                abbr: "JHWH".to_string(),
                expan: "Jahwe".to_string()
            }
        );
    }

    /// adding superfluous elements is irrelevant - this is not always tested
    #[test]
    fn deser_choice_added_text() {
        let xml = r#"<choice>text<abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                xml_lang: None,
                abbr: "JHWH".to_string(),
                expan: "Jahwe".to_string()
            }
        );
    }

    /// changing order is also irrelevant - not always tested
    #[test]
    fn deser_choice_changed_order() {
        let xml = r#"<choice><expan>Jahwe</expan><abbr>JHWH</abbr></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Choice {
                xml_lang: None,
                abbr: "JHWH".to_string(),
                expan: "Jahwe".to_string()
            }
        );
    }

    /// Missing elements lead to errors - not always tested
    #[test]
    fn deser_choice_missing_expan() {
        let xml = r#"<choice><abbr>JHWH</abbr></choice>"#;
        let result: Result<Choice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// base case - everything is here
    #[test]
    fn deser_gap() {
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
    fn deser_gap_no_cert() {
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
    fn deser_gap_allowed_units_correct() {
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
    fn deser_rdg() {
        let xml = r#"<rdg hand="handname" varSeq="1">The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Rdg {
                xml_lang: None,
                hand: Some("handname".to_string()),
                var_seq: 1,
                text: "The content.".to_string(),
            }
        );
    }

    /// rdg - hand is optional
    #[test]
    fn deser_rdg_no_hand() {
        let xml = r#"<rdg varSeq="1">The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Rdg {
                xml_lang: None,
                hand: None,
                var_seq: 1,
                text: "The content.".to_string(),
            }
        );
    }

    /// rdg - varSeq is not optional
    #[test]
    fn deser_rdg_no_varseq() {
        let xml = r#"<rdg>The content.</rdg>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// rdg - text is not optional
    #[test]
    fn deser_rdg_no_text() {
        let xml = r#"<rdg varSeq="3"/>"#;
        let result: Result<Rdg, _> = quick_xml::de::from_str(xml);
        assert!(result.is_err());
    }

    /// app - a list of rdgs
    #[test]
    fn deser_app() {
        let xml = r#"<app><rdg varSeq="1">Content1</rdg><rdg varSeq="2">Content2</rdg></app>"#;
        let result: Result<App, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            App {
                xml_lang: None,
                rdg: vec![
                    Rdg {
                        xml_lang: None,
                        hand: None,
                        var_seq: 1,
                        text: "Content1".to_string()
                    },
                    Rdg {
                        xml_lang: None,
                        hand: None,
                        var_seq: 2,
                        text: "Content2".to_string()
                    }
                ]
            }
        );
    }

    /// damage - base case
    #[test]
    fn deser_damage() {
        let xml = r#"<damage cert="low" agent="water">damaged</damage>"#;
        let result: Result<Damage, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Damage {
                xml_lang: None,
                cert: "low".to_string(),
                agent: "water".to_string(),
                text: "damaged".to_string()
            }
        );
    }

    /// damage - with language
    #[test]
    fn deser_damage_with_lang() {
        let xml = r#"<damage xml:lang="en" cert="low" agent="water">damaged</damage>"#;
        let result: Result<Damage, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Damage {
                xml_lang: Some("en".to_string()),
                cert: "low".to_string(),
                agent: "water".to_string(),
                text: "damaged".to_string()
            }
        );
    }

    /// anchor - base case
    #[test]
    fn deser_anchor() {
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
    fn deser_text_damage_or_choice_text() {
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
    fn deser_text_damage_or_choice_damage() {
        let xml = r#"<damage cert="low" agent="water">damaged</damage>"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TextDamageOrChoice::Damage(Damage {
                xml_lang: None,
                cert: "low".to_string(),
                agent: "water".to_string(),
                text: "damaged".to_string()
            })
        );
    }

    /// TextDamageOrChoice - Choice
    #[test]
    fn deser_text_damage_or_choice_choice() {
        let xml = r#"<choice><abbr>JHWH</abbr><expan>Jahwe</expan></choice>"#;
        let result: Result<TextDamageOrChoice, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            TextDamageOrChoice::Choice(Choice {
                xml_lang: None,
                abbr: "JHWH".to_string(),
                expan: "Jahwe".to_string()
            })
        );
    }

    /// TextDamageOrChoice - Choice
    #[test]
    fn deser_text_damage_or_choice_other() {
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
                xml_lang: None,
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
                xml_lang: None,
                value: TextDamageOrChoice::Damage(Damage {
                    xml_lang: None,
                    cert: "low".to_string(),
                    agent: "water".to_string(),
                    text: "damaged".to_string()
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
                xml_lang: None,
                value: TextDamageOrChoice::Choice(Choice {
                    xml_lang: None,
                    abbr: "JHWH".to_string(),
                    expan: "Jahwe".to_string()
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
                xml_lang: Some("hbo-Hebr-x-babli".to_string()),
                value: TextDamageOrChoice::Choice(Choice {
                    xml_lang: None,
                    abbr: "JHWH".to_string(),
                    expan: "Jahwe".to_string()
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
                xml_lang: None,
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
                xml_lang: Some("grc".to_string()),
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
                xml_lang: None,
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
                        xml_lang: None,
                        value: TextDamageOrChoice::Damage(Damage {
                            xml_lang: None,
                            cert: "low".to_string(),
                            agent: "water".to_string(),
                            text: "damaged".to_string()
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
                xml_lang: Some("hbo-Hebr-x-babli".to_string()),
                n: Some(1),
                div_type: "column".to_string(),
                lines: vec![
                    Line {
                        xml_lang: Some("hbo-Hebr".to_string()),
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
                                xml_lang: None,
                                value: TextDamageOrChoice::Damage(Damage {
                                    xml_lang: None,
                                    cert: "low".to_string(),
                                    agent: "water".to_string(),
                                    text: "damaged".to_string()
                                })
                            })
                        ]
                    },
                    Line {
                        xml_lang: None,
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
                xml_lang: Some("hbo-Hebr-x-babli".to_string()),
                n: Some(1),
                div_type: "column".to_string(),
                lines: vec![Line {
                    xml_lang: None,
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
}
