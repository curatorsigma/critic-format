//! Normalize and Denormalize from types in [`schema`] to those in [`normalized`].

use crate::{normalized, schema};

#[derive(Debug, PartialEq)]
pub enum NormalizationError {
    ColumnNrOverlap(i32),
    ColumnDivIncorrectType(String),
    LineNrOverlap(i32, i32),
    LineDivIncorrectType(String),
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
        }
    }
}
impl core::error::Error for NormalizationError {}

impl TryFrom<schema::Tei> for normalized::Manuscript {
    type Error = NormalizationError;

    fn try_from(value: schema::Tei) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.tei_header.into(),
            text: value.text.try_into()?,
        })
    }
}

impl From<schema::TeiHeader> for normalized::Meta {
    fn from(value: schema::TeiHeader) -> Self {
        Self {
            name: value.file_desc.source_desc.ms_desc.ms_identifier.ms_name,
            page_nr: value.file_desc.source_desc.ms_desc.ms_identifier.page_nr,
            title_stmt: value.file_desc.title_stmt.title,
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
                .summary,
            script_desc: value
                .file_desc
                .source_desc
                .ms_desc
                .phys_desc
                .script_desc
                .summary,
        }
    }
}

impl TryFrom<schema::Text> for normalized::Text {
    type Error = NormalizationError;

    fn try_from(value: schema::Text) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.body.lang,
            columns: try_norm_columns(value.body.columns)?,
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
            text: value.text,
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

#[cfg(test)]
mod test {
    #[test]
    fn complete_normalization() {
        let xml = include_str!("../examples/01_all_elements.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();

        assert!(norm_res.is_ok());
        let expected = crate::normalized::Manuscript {
            meta: crate::normalized::Meta {
                name: "Der Name voms dem Manuskripts".to_string(),
                page_nr: "34 verso".to_string(),
                title_stmt: "Manuskript Name folio 34 verso.".to_string(),
                institution: Some("University of does-not-exist".to_string()),
                collection: Some("Collectors Edition 2 electric boogaloo".to_string()),
                hand_desc: "There are two recognizable Hands: hand1 and hand2.".to_string(),
                script_desc: "Die Schrift in diesem Manuskript gibt es.".to_string(),
            },
            text: crate::normalized::Text {
                lang: Some("hbo-Hebr".to_string()),
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
                                        cert: "high".to_string(),
                                        agent: "water".to_string(),
                                        text: "d".to_string(),
                                    },
                                ),
                                crate::normalized::InlineBlock::Text(
                                    crate::normalized::Paragraph {
                                        lang: None,
                                        content: "erer, wo der Buchstabe nur etwas kaputt ist."
                                            .to_string(),
                                    },
                                ),
                                crate::normalized::InlineBlock::Lacuna(crate::normalized::Lacuna {
                                    reason: "lost".to_string(),
                                    unit: crate::normalized::ExtentUnit::Character,
                                    n: 12,
                                    cert: Some("0.10".to_string()),
                                }),
                                crate::normalized::InlineBlock::Abbreviation(
                                    crate::normalized::Abbreviation {
                                        lang: None,
                                        surface: "JHWH".to_string(),
                                        expansion: "Jahwe".to_string(),
                                    },
                                ),
                                crate::normalized::InlineBlock::Correction(
                                    crate::normalized::Correction {
                                        lang: None,
                                        versions: vec![
                                            crate::normalized::Version {
                                                lang: None,
                                                hand: Some("hand1".to_string()),
                                                text: "sam stuff 1".to_string(),
                                            },
                                            crate::normalized::Version {
                                                lang: None,
                                                hand: Some("hand2".to_string()),
                                                text: "sam stuff 2".to_string(),
                                            },
                                        ],
                                    },
                                ),
                            ],
                        }],
                    },
                ],
            },
        };
        assert_eq!(norm_res.unwrap(), expected);
    }
}
