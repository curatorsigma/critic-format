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

#[derive(Serialize, Deserialize, Debug)]
pub struct Column {
    #[serde(rename = "@lang")]
    pub xml_lang: String,
    #[serde(rename = "@type")]
    pub div_type: String,
    #[serde(rename = "@n")]
    pub n: Option<i32>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub div: Vec<Line>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Line {
    #[serde(rename = "@lang")]
    pub xml_lang: Option<String>,
    #[serde(rename = "@type")]
    pub div_type: String,
    #[serde(rename = "@n")]
    pub n: Option<i32>,
    #[serde(rename = "$value")]
    pub blocks: Vec<InlineBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InlineBlock {
    #[serde(rename = "p")]
    P(TextDamageOrChoice),
    #[serde(rename = "gap")]
    Gap(Gap),
    #[serde(rename = "anchor")]
    Anchor(Anchor),
}

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
}
