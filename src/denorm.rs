//! Normalize and Denormalize from types in [`schema`] to those in [`normalized`].

use crate::{normalized, schema};

#[derive(Debug, PartialEq)]
pub enum NormalizationError {
    ColumnNrOverlap(i32),
    LineNrOverlap(i32, i32),
}
impl core::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::ColumnNrOverlap(x) => {
                write!(f, "The column number {x} would be used multiple times.")
            }
            Self::LineNrOverlap(x, y) => {
                write!(f, "The line number {x} would be used multiple times in column {y}.")
            }
        }
    }
}
impl std::error::Error for NormalizationError {}

impl TryFrom<schema::Text> for normalized::Text {
    type Error = NormalizationError;

    fn try_from(value: schema::Text) -> Result<Self, Self::Error> {
        Ok(Self {
            lang: value.body.lang,
            columns: try_norm_columns(value
                .body
                .columns)?
        })
    }
}

/// Try to normalize a vec of xml-columns
///
/// This can fail because we need to supply the missing column numbers if they were not specified in the
/// xml and we raise errors when there are conflicts (columnnumbers used multiple times etc.)
fn try_norm_columns(columns: Vec<schema::Column>) -> Result<Vec<normalized::Column>, NormalizationError> {
    let mut res = Vec::<normalized::Column>::with_capacity(columns.len());

    let mut next_column_nr = 1;
    for col in columns.into_iter() {
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
        };
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

fn try_norm_lines(lines: Vec<schema::Line>, col_nr: i32) -> Result<Vec<normalized::Line>, NormalizationError> {
    let mut res = Vec::<normalized::Line>::with_capacity(lines.len());

    let mut next_line_nr = 1;
    for line in lines.into_iter() {
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
        };
        let normalized_line = normalized::Line {
            lang: line.lang,
            n: next_line_nr,
            blocks: line.blocks.into_iter().map(|x| <schema::InlineBlock as TryInto<normalized::InlineBlock>>::try_into(x)).collect::<Result<Vec<_>, _>>()?,
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
            schema::InlineBlock::P(x) => {
                match x.value {
                    schema::TextDamageOrChoice::Damage(y) => {
                        normalized::InlineBlock::Uncertain(y)
                    }
                    schema::TextDamageOrChoice::Text(y) => {
                        normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: x.lang,
                            content: y,
                        })
                    }
                    schema::TextDamageOrChoice::Choice(y) => {
                        normalized::InlineBlock::Abbreviation(y)
                    }
                }
            }
            schema::InlineBlock::Gap(x) => {
                normalized::InlineBlock::Lacuna(x)
            }
            schema::InlineBlock::Anchor(x) => {
                normalized::InlineBlock::Anchor(x.into())
            }
            schema::InlineBlock::App(x) => {
                normalized::InlineBlock::Correction(x.into())
            }
        })
    }
}

impl From<schema::App> for normalized::Correction {
    fn from(value: schema::App) -> Self {
        Self {
            lang: value.lang,
            versions: value.rdg.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl From<schema::Rdg> for normalized::Version {
    fn from(value: schema::Rdg) -> Self {
        Self { lang: value.lang, hand: value.hand, text: value.text }
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
