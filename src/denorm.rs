//! Normalize and Denormalize from types in [`schema`] to those in [`normalized`].

use crate::{normalized, schema};

/// This publication statement MUST be present in every xml file and this is enforced.
pub const PUBLICATION_STATEMENT: &str = "This digital reproduction is published as part of TanakhCC and licensed as https://creativecommons.org/publicdomain/zero/1.0.";

/// An error while Normalizing or Denormalizing a document.
#[derive(Debug, PartialEq)]
pub enum NormalizationError {
    /// This column number would overlap and be used twice
    ColumnNrOverlap(i32),
    /// A div is required to specify a column but its `@type` is not `column`
    ///
    /// Argument is the `@type` actually present
    ColumnDivIncorrectType(String),
    /// This line number would overlap and be used twice
    ///
    /// Arguments are
    /// - the line number
    /// - the column this line is in
    LineNrOverlap(i32, i32),
    /// A div is required to specify a line but its `@type` is not `line`
    ///
    /// Argument is the `@type` actually present
    LineDivIncorrectType(String),
    /// The publication statement does not match the one given in [`PUBLICATION_STATEMENT`]
    PublicationStmtIncorrect,
    /// The normalized version had more then 2^32 - 1 versions for a single correction and is thus not
    /// representable
    TooManyVersions,
    /// The body element of the xml file has not `@xml:lang` set
    NoDefaultLanguage,
    /// Two pages were in the wrong order.
    ///
    /// last page - this page
    PageNumbersNotOrdered(String, String),
}
impl core::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::ColumnNrOverlap(x) => {
                write!(f, "The column number {x} would be used multiple times.")
            }
            Self::ColumnDivIncorrectType(x) => {
                write!(
                    f,
                    "A div that should represent a column hat incorrect type {x}. Must be \"column\"."
                )
            }
            Self::LineNrOverlap(x, y) => {
                write!(
                    f,
                    "The line number {x} would be used multiple times in column {y}."
                )
            }
            Self::LineDivIncorrectType(x) => {
                write!(
                    f,
                    "A div that should represent a line hat incorrect type {x}. Must be \"line\"."
                )
            }
            Self::PublicationStmtIncorrect => {
                write!(
                    f,
                    "The publicationStmt was not exactly \"{PUBLICATION_STATEMENT}\"."
                )
            }
            Self::TooManyVersions => {
                write!(
                    f,
                    "There were more then 2^32 - 1 versions for one correction. Are you okay?"
                )
            }
            Self::NoDefaultLanguage => {
                write!(f, "The xml text body has no \"@xml:lang\" set.")
            }
            Self::PageNumbersNotOrdered(last, this) => {
                write!(
                    f,
                    "The two pages {last} and {this} are in the wrong order or have the same name."
                )
            }
        }
    }
}
impl core::error::Error for NormalizationError {}

impl TryFrom<schema::Tei> for normalized::Manuscript {
    type Error = NormalizationError;

    fn try_from(value: schema::Tei) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        Ok(Self {
            meta: trimmed.tei_header.try_into()?,
            text: trimmed.text.try_into()?,
        })
    }
}

impl TryFrom<schema::TeiHeader> for normalized::Meta {
    type Error = NormalizationError;

    fn try_from(value: schema::TeiHeader) -> Result<Self, Self::Error> {
        if value.file_desc.publication_stmt.p != *PUBLICATION_STATEMENT {
            return Err(NormalizationError::PublicationStmtIncorrect);
        }
        Ok(Self {
            alt_identifier: value
                .file_desc
                .source_desc
                .ms_desc
                .ms_identifier
                .alt_identifier
                .into_iter()
                .map(|a| a.idno.name)
                .collect(),
            title: value.file_desc.title_stmt.title,
            institution: value
                .file_desc
                .source_desc
                .ms_desc
                .ms_identifier
                .institution,
            collection: value.file_desc.source_desc.ms_desc.ms_identifier.collection,
            hand_desc: value
                .file_desc
                .source_desc
                .ms_desc
                .phys_desc
                .hand_desc
                .map(|d| d.summary),
            script_desc: value
                .file_desc
                .source_desc
                .ms_desc
                .phys_desc
                .script_desc
                .map(|d| d.summary),
        })
    }
}

impl TryFrom<schema::Text> for normalized::Text {
    type Error = NormalizationError;

    fn try_from(value: schema::Text) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value
                .body
                .lang
                .ok_or(NormalizationError::NoDefaultLanguage)?,
            pages: try_norm_pages(value.body.pages)?,
        })
    }
}

/// Try to normalize pages
///
/// This can fail because two pages may be in the wrong order in the [`schema`] version, but this
/// is no longer allowed in the [`normalized`] form.
fn try_norm_pages(pages: Vec<schema::Page>) -> Result<Vec<normalized::Page>, NormalizationError> {
    let mut res = Vec::<normalized::Page>::with_capacity(pages.len());

    let mut last_name = None;
    for page in pages {
        if let Some(l) = last_name {
            if l >= page.n {
                return Err(NormalizationError::PageNumbersNotOrdered(l, page.n));
            };
        };
        last_name = Some(page.n.clone());
        res.push(page.try_into()?);
    }
    Ok(res)
}

impl TryFrom<schema::Page> for normalized::Page {
    type Error = NormalizationError;

    fn try_from(value: schema::Page) -> Result<Self, Self::Error> {
        Ok(Self {
            n: value.n,
            lang: value.lang,
            columns: try_norm_columns(value.columns)?,
        })
    }
}

/// Try to normalize a vec of xml-columns
///
/// This can fail because we need to supply the missing column numbers if they were not specified in the
/// xml and we raise errors when there are conflicts (columnnumbers used multiple times etc.)
fn try_norm_columns(
    columns: Vec<schema::Column>,
) -> Result<Vec<normalized::Column>, NormalizationError> {
    let mut res = Vec::<normalized::Column>::with_capacity(columns.len());

    let mut next_column_nr = 1;
    for col in columns {
        // make sure that the divtype was actually set to column
        if col.div_type != "column" {
            return Err(NormalizationError::ColumnDivIncorrectType(col.div_type));
        }
        // if a column number is given and it is consistent, use it instead of the auto-increment
        if let Some(x) = col.n {
            // because gaps can include entire columns, the next column might be offset more then
            // one
            if x >= next_column_nr {
                next_column_nr = x;
            } else {
                // we cannot decrement the next_column_nr, this would mean a column-nr is used
                // multiple times
                return Err(NormalizationError::ColumnNrOverlap(x));
            }
        }
        let normalized_col = normalized::Column {
            lang: col.lang,
            n: next_column_nr,
            lines: try_norm_lines(col.lines, next_column_nr)?,
        };
        // now auto-increment to the next column
        next_column_nr += 1;
        res.push(normalized_col);
    }
    Ok(res)
}

fn try_norm_lines(
    lines: Vec<schema::Line>,
    col_nr: i32,
) -> Result<Vec<normalized::Line>, NormalizationError> {
    let mut res = Vec::<normalized::Line>::with_capacity(lines.len());

    let mut next_line_nr = 1;
    for line in lines {
        // make sure that the divtype was actually set to line
        if line.div_type != "line" {
            return Err(NormalizationError::LineDivIncorrectType(line.div_type));
        }

        // if a line number is given and it is consistent, use it instead of the auto-increment
        if let Some(x) = line.n {
            // because gaps can include entire lines, the next line might be offset more then
            // one
            if x >= next_line_nr {
                next_line_nr = x;
            } else {
                // we cannot decrement the next_line_nr, this would mean a line-nr is used
                // multiple times
                return Err(NormalizationError::LineNrOverlap(x, col_nr));
            }
        }

        let normalized_line = normalized::Line {
            lang: line.lang,
            n: next_line_nr,
            blocks: line
                .blocks
                .into_iter()
                .map(<schema::InlineBlock as TryInto<normalized::InlineBlock>>::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        };
        // now auto-increment to the next line
        next_line_nr += 1;
        res.push(normalized_line);
    }
    Ok(res)
}

impl TryFrom<schema::InlineBlock> for normalized::InlineBlock {
    type Error = NormalizationError;

    fn try_from(value: schema::InlineBlock) -> Result<Self, Self::Error> {
        Ok(match value {
            schema::InlineBlock::P(x) => match x.value {
                schema::TextDamageOrChoice::Damage(y) => normalized::InlineBlock::Uncertain(y),
                schema::TextDamageOrChoice::Text(y) => {
                    normalized::InlineBlock::Text(normalized::Paragraph {
                        lang: x.lang,
                        content: y,
                    })
                }
                schema::TextDamageOrChoice::Choice(y) => normalized::InlineBlock::Abbreviation(y),
            },
            schema::InlineBlock::Gap(x) => normalized::InlineBlock::Lacuna(x),
            schema::InlineBlock::Anchor(x) => normalized::InlineBlock::Anchor(x.into()),
            schema::InlineBlock::App(x) => normalized::InlineBlock::Correction(x.into()),
            schema::InlineBlock::Space(x) => normalized::InlineBlock::Space(x),
        })
    }
}

impl From<schema::App> for normalized::Correction {
    fn from(value: schema::App) -> Self {
        Self {
            lang: value.lang,
            versions: value
                .rdg
                .into_iter()
                .map(core::convert::Into::into)
                .collect(),
        }
    }
}

impl From<schema::Rdg> for normalized::Version {
    fn from(value: schema::Rdg) -> Self {
        Self {
            lang: value.lang,
            hand: value.hand,
            content: value.content,
        }
    }
}

impl From<schema::Anchor> for normalized::Anchor {
    fn from(value: schema::Anchor) -> Self {
        Self {
            anchor_id: value.xml_id,
            anchor_type: value.anchor_type,
        }
    }
}

// denormalizing (from normalized form to schema-form)

impl TryFrom<normalized::Manuscript> for schema::Tei {
    type Error = NormalizationError;

    fn try_from(value: normalized::Manuscript) -> Result<Self, Self::Error> {
        Ok(Self {
            xmlns: "http://www.tei-c.org/ns/1.0".to_string(),
            tei_header: value.meta.into(),
            text: value.text.try_into()?,
        })
    }
}

impl From<normalized::Meta> for schema::TeiHeader {
    fn from(value: normalized::Meta) -> Self {
        Self {
            file_desc: schema::FileDesc {
                title_stmt: schema::TitleStmt { title: value.title },
                publication_stmt: schema::PublicationStmt {
                    p: PUBLICATION_STATEMENT.to_string(),
                },
                source_desc: schema::SourceDesc {
                    ms_desc: schema::MsDesc {
                        ms_identifier: schema::MsIdentifier {
                            institution: value.institution,
                            collection: value.collection,
                            alt_identifier: value
                                .alt_identifier
                                .into_iter()
                                .map(|a| schema::AltIdentifier {
                                    idno: schema::IdNo { name: a },
                                })
                                .collect(),
                        },
                        phys_desc: schema::PhysDesc {
                            hand_desc: value.hand_desc.map(|d| schema::HandDesc { summary: d }),
                            script_desc: value
                                .script_desc
                                .map(|d| schema::ScriptDesc { summary: d }),
                        },
                    },
                },
            },
        }
    }
}

impl TryFrom<normalized::Text> for schema::Text {
    type Error = NormalizationError;
    fn try_from(value: normalized::Text) -> Result<Self, Self::Error> {
        Ok(Self {
            body: schema::Body {
                lang: Some(value.lang),
                pages: value
                    .pages
                    .into_iter()
                    .map(core::convert::TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
    }
}

impl TryFrom<normalized::Page> for schema::Page {
    type Error = NormalizationError;
    fn try_from(value: normalized::Page) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.lang,
            n: value.n,
            div_type: "page".to_string(),
            columns: value
                .columns
                .into_iter()
                .map(|c| c.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<normalized::Column> for schema::Column {
    type Error = NormalizationError;

    fn try_from(value: normalized::Column) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.lang,
            div_type: "column".to_string(),
            n: Some(value.n),
            lines: value
                .lines
                .into_iter()
                .map(core::convert::TryInto::<schema::Line>::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<normalized::Line> for schema::Line {
    type Error = NormalizationError;

    fn try_from(value: normalized::Line) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.lang,
            div_type: "line".to_string(),
            n: Some(value.n),
            blocks: value
                .blocks
                .into_iter()
                .map(core::convert::TryInto::<schema::InlineBlock>::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<normalized::InlineBlock> for schema::InlineBlock {
    type Error = NormalizationError;

    fn try_from(value: normalized::InlineBlock) -> Result<Self, Self::Error> {
        Ok(match value {
            normalized::InlineBlock::Text(x) => schema::InlineBlock::P(schema::TDOCWrapper {
                lang: x.lang.clone(),
                value: schema::TextDamageOrChoice::Text(x.content),
            }),
            normalized::InlineBlock::Lacuna(x) => schema::InlineBlock::Gap(x),
            normalized::InlineBlock::Uncertain(x) => schema::InlineBlock::P(schema::TDOCWrapper {
                lang: x.lang.clone(),
                value: schema::TextDamageOrChoice::Damage(x),
            }),
            normalized::InlineBlock::Abbreviation(x) => {
                schema::InlineBlock::P(schema::TDOCWrapper {
                    lang: x.lang.clone(),
                    value: schema::TextDamageOrChoice::Choice(x),
                })
            }
            normalized::InlineBlock::Anchor(x) => schema::InlineBlock::Anchor(x.into()),
            normalized::InlineBlock::Correction(x) => schema::InlineBlock::App(x.try_into()?),
            normalized::InlineBlock::Space(x) => schema::InlineBlock::Space(x),
        })
    }
}

impl From<normalized::Anchor> for schema::Anchor {
    fn from(value: normalized::Anchor) -> Self {
        Self {
            xml_id: value.anchor_id,
            anchor_type: value.anchor_type,
        }
    }
}

impl TryFrom<normalized::Correction> for schema::App {
    type Error = NormalizationError;

    fn try_from(value: normalized::Correction) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.lang,
            rdg: denorm_versions(value.versions)?,
        })
    }
}

fn denorm_versions(
    versions: Vec<normalized::Version>,
) -> Result<Vec<schema::Rdg>, NormalizationError> {
    let mut res = Vec::with_capacity(versions.len());
    for (i, version) in versions.into_iter().enumerate() {
        res.push(schema::Rdg {
            lang: version.lang,
            hand: version.hand,
            content: version.content,
            var_seq: i32::try_from(i).map_err(|_| NormalizationError::TooManyVersions)?,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    /// test the entire normalization procedure - there is not a lot going on here that is not
    /// trivial, so there are few test here
    ///
    /// This also tests automatic trimming
    #[test]
    fn complete_normalization() {
        let xml = include_str!("../examples/01_all_elements.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();

        assert!(norm_res.is_ok());
        let expected = crate::normalized::Manuscript {
            meta: crate::normalized::Meta {
                alt_identifier: vec![],
                title: "Manuskript Name".to_string(),
                institution: Some("University of does-not-exist".to_string()),
                collection: Some("Collectors Edition 2 electric boogaloo".to_string()),
                hand_desc: Some("There are two recognizable Hands: hand1 and hand2.".to_string()),
                script_desc: Some("Die Schrift in diesem Manuskript gibt es.".to_string()),
            },
            text: crate::normalized::Text {
                lang: "hbo-Hebr".to_string(),
                pages: vec![crate::normalized::Page {
                    lang: None,
                    n: "34_v".to_string(),
                    columns: vec![
                        crate::normalized::Column {
                            lang: None,
                            n: 1,
                            lines: vec![
                                crate::normalized::Line {
                                    lang: None,
                                    n: 2,
                                    blocks: vec![
                                        crate::normalized::InlineBlock::Text(
                                            crate::normalized::Paragraph {
                                                lang: None,
                                                content: "asdfa".to_string(),
                                            },
                                        ),
                                        crate::normalized::InlineBlock::Anchor(
                                            crate::normalized::Anchor {
                                                anchor_id: "A_V_MT_1Kg-3-4".to_string(),
                                                anchor_type: "Masoretic".to_string(),
                                            },
                                        ),
                                        crate::normalized::InlineBlock::Anchor(
                                            crate::normalized::Anchor {
                                                anchor_id: "A_V_LXX_1Kg-3-4".to_string(),
                                                anchor_type: "Septuagint".to_string(),
                                            },
                                        ),
                                        crate::normalized::InlineBlock::Text(
                                            crate::normalized::Paragraph {
                                                lang: None,
                                                content: "sdfsa".to_string(),
                                            },
                                        ),
                                    ],
                                },
                                crate::normalized::Line {
                                    lang: Some("hbo-Hebr-x-babli".to_string()),
                                    n: 3,
                                    blocks: vec![crate::normalized::InlineBlock::Text(
                                        crate::normalized::Paragraph {
                                            lang: None,
                                            content: "Some stuff with babylonian Niqud".to_string(),
                                        },
                                    )],
                                },
                            ],
                        },
                        crate::normalized::Column {
                            lang: None,
                            n: 2,
                            lines: vec![crate::normalized::Line {
                                lang: None,
                                n: 2,
                                blocks: vec![
                                    crate::normalized::InlineBlock::Text(
                                        crate::normalized::Paragraph {
                                            lang: None,
                                            content: "Hier ein an".to_string(),
                                        },
                                    ),
                                    crate::normalized::InlineBlock::Uncertain(
                                        crate::normalized::Uncertain {
                                            lang: None,
                                            cert: Some("high".to_string()),
                                            agent: "water".to_string(),
                                            content: "d".to_string(),
                                        },
                                    ),
                                    crate::normalized::InlineBlock::Text(
                                        crate::normalized::Paragraph {
                                            lang: None,
                                            content: "erer, wo der Buchstabe nur etwas kaputt ist."
                                                .to_string(),
                                        },
                                    ),
                                    crate::normalized::InlineBlock::Lacuna(
                                        crate::normalized::Lacuna {
                                            reason: "lost".to_string(),
                                            unit: crate::normalized::ExtentUnit::Character,
                                            n: 12,
                                            cert: Some("0.10".to_string()),
                                        },
                                    ),
                                    crate::normalized::InlineBlock::Abbreviation(
                                        crate::normalized::Abbreviation {
                                            lang: None,
                                            surface: crate::schema::AbbrSurface {
                                                lang: None,
                                                content: "JHWH".to_string(),
                                            },
                                            expansion: crate::schema::AbbrExpansion {
                                                lang: None,
                                                content: "Jahwe".to_string(),
                                            },
                                        },
                                    ),
                                    crate::normalized::InlineBlock::Correction(
                                        crate::normalized::Correction {
                                            lang: None,
                                            versions: vec![
                                                crate::normalized::Version {
                                                    lang: None,
                                                    hand: Some("hand1".to_string()),
                                                    content: "sam stuff 1".to_string(),
                                                },
                                                crate::normalized::Version {
                                                    lang: None,
                                                    hand: Some("hand2".to_string()),
                                                    content: "sam stuff 2".to_string(),
                                                },
                                            ],
                                        },
                                    ),
                                ],
                            }],
                        },
                    ],
                }],
            },
        };
        assert_eq!(norm_res.unwrap(), expected);
    }

    /// normalization after denormalization is the identity
    ///
    /// Note that the other direction is not correct:
    /// not every input normalizes, so denorm circ norm cannot be the identity
    ///
    /// Technically denormalizing is also fallible, but these errors can be ignored
    #[test]
    fn norm_circ_denorm_is_identity() {
        let xml = include_str!("../examples/01_all_elements.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let normed = norm_res.unwrap();

        // now test that denorming and then norming is the identity
        let denormed: crate::schema::Tei = normed.clone().try_into().unwrap();
        let renormed: crate::normalized::Manuscript = denormed.try_into().unwrap();
        assert_eq!(renormed, normed);
    }
}
