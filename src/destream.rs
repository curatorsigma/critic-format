//! Converting from [`normalized`](crate::normalized) to [`streamed`](crate::streamed)
//! representation.
//!
//! In particular, insert or extract line and column breaks and create the corresponding hierarchy
//! of blocks in the destreamed version.

use std::collections::HashMap;

use crate::normalized;
use crate::streamed;

/// An error while Normalizing or Denormalizing a document.
#[derive(Debug, PartialEq)]
pub enum StreamError {
    /// While starting work on a new column, the index should be `arg_1` but is actually `arg_2`
    ColumnIndexInconsistent(i32, i32),
    /// While starting work on a new line, the index should be `arg_1` but is actually `arg_2`
    LineIndexInconsistent(i32, i32),
    /// No block in the streamed form has a language associated with it, so we cannot choose the
    /// default language for the text
    NoBlockWithLanguage,
}
impl core::fmt::Display for StreamError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::ColumnIndexInconsistent(expect, actual) => {
                write!(
                    f,
                    "The next column index should be {expect} but is set to {actual}."
                )
            }
            Self::LineIndexInconsistent(expect, actual) => {
                write!(
                    f,
                    "The next line index should be {expect} but is set to {actual}."
                )
            }
            Self::NoBlockWithLanguage => {
                write!(
                    f,
                    "No block in the streamed form has a lanaguage associated with it, so we cannot choose the default language."
                )
            }
        }
    }
}
impl core::error::Error for StreamError {}

impl TryFrom<normalized::Manuscript> for streamed::Manuscript {
    type Error = StreamError;

    fn try_from(value: normalized::Manuscript) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta,
            content: value.text.try_into()?,
        })
    }
}

/// This does two main things:
/// - unroll the hierarchy into a stream, inserting line and column breaks
/// - Associate the correct language to every Block
impl TryFrom<normalized::Text> for Vec<streamed::Block> {
    type Error = StreamError;

    fn try_from(value: normalized::Text) -> Result<Self, Self::Error> {
        let mut res = Vec::with_capacity(
            value
                .columns
                .iter()
                .map(|c| c.lines.iter().map(|l| l.blocks.len()).sum::<usize>())
                .sum(),
        );

        let mut current_language: String;
        // the index of the column in logical ordering (i.e. getting larger when passing a
        // column-spaning lacuna)
        let mut col_idx = 1;
        let num_of_cols = value.columns.len();
        'col: for (defined_c_idx, col) in value.columns.into_iter().enumerate() {
            if col.n != col_idx {
                return Err(StreamError::ColumnIndexInconsistent(col_idx, col.n));
            }
            current_language = if let Some(new_lang) = col.lang {
                new_lang
            } else {
                value.lang.clone()
            };
            let column_lang = current_language;
            let num_of_lines = col.lines.len();
            let mut line_idx = 1;
            'line: for (defined_l_idx, line) in col.lines.into_iter().enumerate() {
                if line.n != line_idx {
                    return Err(StreamError::LineIndexInconsistent(line_idx, line.n));
                }
                current_language = if let Some(new_lang) = line.lang {
                    new_lang
                } else {
                    column_lang.clone()
                };
                let line_lang = current_language.clone();
                for block in line.blocks {
                    current_language = if let Some(new_lang) = block.language() {
                        new_lang.to_string()
                    } else {
                        line_lang.clone()
                    };
                    let streamed_block = (current_language.clone(), block).try_into()?;
                    // break off if we start a multi-line or multi-column gap with this block
                    match streamed_block {
                        // a lacuna spanning multiple lines.
                        // we increment the line-nr by the appropriate amount and continue
                        // streaming from the next line defined in the xml
                        //
                        // There are n lines skipped, so the next defined line has index +n (from
                        // skipped lines) +1 (from this line ending) relative to the current line
                        // the `<gap>` is on.
                        //
                        // (the other lines are NOT to be defined in the xml, since they are taken
                        // up by the `<gap>`)
                        streamed::Block::Lacuna(streamed::Lacuna {
                            unit: normalized::ExtentUnit::Line,
                            n,
                            ..
                        }) => {
                            line_idx += n + 1;
                            res.push(streamed_block);
                            continue 'line;
                        }
                        // a lacuna spanning multiple columns.
                        // we increment the column-nr by the appropriate amount and continue
                        // streaming from the next column defined in the xml
                        streamed::Block::Lacuna(streamed::Lacuna {
                            unit: normalized::ExtentUnit::Column,
                            n,
                            ..
                        }) => {
                            col_idx += n + 1;
                            res.push(streamed_block);
                            continue 'col;
                        }
                        _ => {}
                    }
                    res.push(streamed_block);
                }
                // the line is now ended - insert a line break except for the last line
                if defined_l_idx + 1 < num_of_lines {
                    res.push(streamed::Block::Break(streamed::BreakType::Line));
                }
                line_idx += 1;
            }
            // the column is now ended - insert a column break except for the last column
            if defined_c_idx + 1 < num_of_cols {
                res.push(streamed::Block::Break(streamed::BreakType::Column));
            }
            col_idx += 1;
        }

        // The last element is always a columnbreak, which we drop again

        Ok(res)
    }
}

impl TryFrom<(String, normalized::InlineBlock)> for streamed::Block {
    type Error = StreamError;

    fn try_from(value: (String, normalized::InlineBlock)) -> Result<Self, Self::Error> {
        Ok(match value.1 {
            normalized::InlineBlock::Text(x) => streamed::Block::Text(streamed::Paragraph {
                lang: if let Some(p_lang) = x.lang {
                    p_lang
                } else {
                    value.0
                },
                content: x.content,
            }),
            normalized::InlineBlock::Uncertain(x) => {
                streamed::Block::Uncertain((value.0, x).into())
            }
            normalized::InlineBlock::Lacuna(x) => streamed::Block::Lacuna(x),
            normalized::InlineBlock::Anchor(x) => streamed::Block::Anchor(x),
            normalized::InlineBlock::Correction(x) => {
                streamed::Block::Correction((value.0, x).into())
            }
            normalized::InlineBlock::Abbreviation(x) => {
                streamed::Block::Abbreviation((value.0, x).into())
            }
        })
    }
}

impl From<(String, normalized::Uncertain)> for streamed::Uncertain {
    fn from(value: (String, normalized::Uncertain)) -> Self {
        Self {
            lang: if let Some(u_lang) = value.1.lang {
                u_lang
            } else {
                value.0
            },
            cert: value.1.cert,
            agent: value.1.agent,
            content: value.1.content,
        }
    }
}

impl From<(String, normalized::Correction)> for streamed::Correction {
    fn from(value: (String, normalized::Correction)) -> Self {
        Self {
            versions: core::iter::repeat(value.0)
                .zip(value.1.versions)
                .map(core::convert::Into::into)
                .collect(),
        }
    }
}

impl From<(String, normalized::Version)> for streamed::Version {
    fn from(value: (String, normalized::Version)) -> Self {
        Self {
            lang: if let Some(inner_lang) = value.1.lang {
                inner_lang
            } else {
                value.0
            },
            hand: value.1.hand,
            content: value.1.content,
        }
    }
}

impl From<(String, normalized::Abbreviation)> for streamed::Abbreviation {
    fn from(value: (String, normalized::Abbreviation)) -> Self {
        // default language is value.0
        // language set on the entire normalized abbreviation is value.1.lang (OPTION)
        // language set on the individual arms are value.1.surface.lang, etc.
        let surface_lang = if let Some(n_surface_lang) = value.1.surface.lang {
            n_surface_lang
        } else if let Some(ref n_abbr_lang) = value.1.lang {
            n_abbr_lang.clone()
        } else {
            value.0.clone()
        };
        let expansion_lang = if let Some(n_expansion_lang) = value.1.expansion.lang {
            n_expansion_lang
        } else if let Some(n_abbr_lang) = value.1.lang {
            n_abbr_lang
        } else {
            value.0.clone()
        };
        Self {
            surface_lang,
            expansion_lang,
            surface: value.1.surface.content,
            expansion: value.1.expansion.content,
        }
    }
}

// destream - turn a stream into normalized form

impl TryFrom<streamed::Manuscript> for normalized::Manuscript {
    type Error = StreamError;

    fn try_from(value: streamed::Manuscript) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta,
            text: value.content.try_into()?,
        })
    }
}

impl TryFrom<Vec<streamed::Block>> for normalized::Text {
    type Error = StreamError;

    fn try_from(value: Vec<streamed::Block>) -> Result<Self, Self::Error> {
        let mut language_use = HashMap::<String, i32>::new();
        let mut columns = Vec::<normalized::Column>::new();
        let mut column_idx = 1;
        let mut language_use_in_col = HashMap::<String, i32>::new();
        let mut lines = Vec::<normalized::Line>::new();
        let mut line_idx = 1;
        let mut language_use_in_line = HashMap::<String, i32>::new();
        let mut blocks_in_line = Vec::<normalized::InlineBlock>::new();

        for block in value {
            // update language use
            if let Some(lang_in_this_block) = block.language() {
                // global
                if let Some(this_lang_val) = language_use.get_mut(lang_in_this_block) {
                    *this_lang_val += 1;
                } else {
                    language_use.insert(lang_in_this_block.to_string(), 1);
                }
                // column
                if let Some(this_lang_val) = language_use_in_col.get_mut(lang_in_this_block) {
                    *this_lang_val += 1;
                } else {
                    language_use_in_col.insert(lang_in_this_block.to_string(), 1);
                }
                // line
                if let Some(this_lang_val) = language_use_in_line.get_mut(lang_in_this_block) {
                    *this_lang_val += 1;
                } else {
                    language_use_in_line.insert(lang_in_this_block.to_string(), 1);
                }
            }

            // add this block to this line - but there are a few exceptions:
            handle_block(
                block,
                &mut blocks_in_line,
                &mut lines,
                &mut columns,
                &mut language_use_in_line,
                &mut language_use_in_col,
                &mut line_idx,
                &mut column_idx,
            );
        }

        // now we need to add the remaining blocks as a final line/column as in a column break
        if !blocks_in_line.is_empty() {
            // first end the line
            let most_common_lang_in_line = language_use_in_line
                .iter()
                .max_by(|a, b| a.1.cmp(b.1))
                .map(|(k, _v)| k)
                .map(std::string::ToString::to_string);
            lines.push(normalized::Line {
                lang: most_common_lang_in_line,
                n: line_idx,
                blocks: core::mem::take(&mut blocks_in_line),
            });
        }
        if !lines.is_empty() {
            // now end the column and go to the next one
            let most_common_lang_in_col = language_use_in_col
                .iter()
                .max_by(|a, b| a.1.cmp(b.1))
                .map(|(k, _v)| k)
                .map(std::string::ToString::to_string);
            let take_lines = core::mem::take(&mut lines);
            columns.push(normalized::Column {
                lang: most_common_lang_in_col,
                n: column_idx,
                lines: take_lines,
            });
        }

        let most_common_lang = normalize_language(&mut columns, &language_use)?;

        Ok(Self {
            lang: most_common_lang.to_string(),
            columns,
        })
    }
}

fn most_common_lang(lang_context: &HashMap<String, i32>) -> Option<&str> {
    lang_context
        .iter()
        .max_by(|a, b| a.1.cmp(b.1))
        .map(|(k, _v)| k.as_str())
}

/// End a line with the current `blocks_in_line`, push it to the lines, increase the index to the
/// next line, clear language use in the line
fn end_line(
    lines: &mut Vec<normalized::Line>,
    blocks_in_line: Vec<normalized::InlineBlock>,
    line_idx: &mut i32,
    language_use_in_line: &mut HashMap<String, i32>,
) {
    lines.push(normalized::Line {
        lang: most_common_lang(language_use_in_line).map(std::string::ToString::to_string),
        n: *line_idx,
        blocks: blocks_in_line,
    });
    *language_use_in_line = HashMap::<String, i32>::new();
    *line_idx += 1;
}

/// End a column with the current `lines`, push it to the columns, increase the index to the
/// next column, clear language use in the column, reset the line index to 1
fn end_column(
    columns: &mut Vec<normalized::Column>,
    lines: Vec<normalized::Line>,
    line_idx: &mut i32,
    column_idx: &mut i32,
    language_use_in_col: &mut HashMap<String, i32>,
) {
    columns.push(normalized::Column {
        lang: most_common_lang(language_use_in_col).map(std::string::ToString::to_string),
        n: *column_idx,
        lines,
    });
    *language_use_in_col = HashMap::<String, i32>::new();
    *column_idx += 1;
    *line_idx = 1;
}

/// Add a block to the datastructure, update language use and forward line and column indexes when
/// a line or column is ended by this block
#[allow(clippy::too_many_arguments)]
fn handle_block(
    block: streamed::Block,
    blocks_in_line: &mut Vec<normalized::InlineBlock>,
    lines: &mut Vec<normalized::Line>,
    columns: &mut Vec<normalized::Column>,
    language_use_in_line: &mut HashMap<String, i32>,
    language_use_in_col: &mut HashMap<String, i32>,
    line_idx: &mut i32,
    column_idx: &mut i32,
) {
    match block {
        // end this line, start a new one
        streamed::Block::Break(streamed::BreakType::Line) => {
            end_line(
                lines,
                core::mem::take(blocks_in_line),
                line_idx,
                language_use_in_line,
            );
        }
        // end this column, start a new one
        streamed::Block::Break(streamed::BreakType::Column) => {
            // first end the line
            end_line(
                lines,
                core::mem::take(blocks_in_line),
                line_idx,
                language_use_in_line,
            );

            // now end the column and go to the next one
            end_column(
                columns,
                core::mem::take(lines),
                line_idx,
                column_idx,
                language_use_in_col,
            );
        }
        // end this line, skip several, start a new one
        streamed::Block::Lacuna(
            l @ streamed::Lacuna {
                unit: streamed::ExtentUnit::Line,
                n: extent,
                ..
            },
        ) => {
            // first push the lacuna itself as a block
            blocks_in_line.push(normalized::InlineBlock::Lacuna(l));
            // then end the line
            end_line(
                lines,
                core::mem::take(blocks_in_line),
                line_idx,
                language_use_in_line,
            );
            // skip `extent` lines
            *line_idx += extent;
        }
        // end this column, skip several, start a new one
        streamed::Block::Lacuna(
            l @ streamed::Lacuna {
                unit: streamed::ExtentUnit::Column,
                n: extent,
                ..
            },
        ) => {
            // first push the lacuna itself as a block
            blocks_in_line.push(normalized::InlineBlock::Lacuna(l));
            // then end the line
            end_line(
                lines,
                core::mem::take(blocks_in_line),
                line_idx,
                language_use_in_line,
            );

            // and finally end the column, skip some and and go to the next one
            end_column(
                columns,
                core::mem::take(lines),
                line_idx,
                column_idx,
                language_use_in_col,
            );
            *column_idx += extent;
        }
        // these are the normal blocks - just convert them
        streamed::Block::Text(x) => {
            blocks_in_line.push(normalized::InlineBlock::Text(x.into()));
        }
        streamed::Block::Lacuna(l) => {
            blocks_in_line.push(normalized::InlineBlock::Lacuna(l));
        }
        streamed::Block::Uncertain(x) => {
            blocks_in_line.push(normalized::InlineBlock::Uncertain(x.into()));
        }
        streamed::Block::Anchor(x) => {
            blocks_in_line.push(normalized::InlineBlock::Anchor(x));
        }
        streamed::Block::Correction(x) => {
            blocks_in_line.push(normalized::InlineBlock::Correction(x.into()));
        }
        streamed::Block::Abbreviation(x) => {
            blocks_in_line.push(normalized::InlineBlock::Abbreviation(x.into()));
        }
    }
}

fn normalize_language<'b>(
    columns: &mut Vec<normalized::Column>,
    language_use: &'b HashMap<String, i32>,
) -> Result<&'b str, StreamError> {
    // calculate the language most commonly used in this text
    let most_common_lang = language_use
        .iter()
        .max_by(|a, b| a.1.cmp(b.1))
        .map(|(k, _v)| k)
        .ok_or(StreamError::NoBlockWithLanguage)?;

    // we now have a completely destreamed version
    // However, all leaf nodes (Text, Uncertain, ...) have the language explicitly set, which
    // is wasteful and unusual for xml.
    //
    // We now iterate top-down through the hierarchy and set the language
    // Note that the language is set on all non-leafs (column/line) where:
    // - any leaf has a lang attribute
    // - the value is the most common language in the leafs below it
    let global_lang = most_common_lang;

    for col in columns {
        let column_lang = col.lang.clone();
        // unset the columns language if it is equal to the global language
        if col.lang.as_ref().is_some_and(|x| x == global_lang) {
            col.lang = None;
        }
        for line in &mut col.lines {
            let line_lang = line.lang.clone();
            // unset the line language if it is equal to the column language
            if line.lang == column_lang {
                line.lang = None;
            }
            // unset the block language if it is the same as line lang
            for block in &mut line.blocks {
                match block {
                    normalized::InlineBlock::Lacuna(_) | normalized::InlineBlock::Anchor(_) => {}
                    normalized::InlineBlock::Text(x) => {
                        if x.lang == line_lang {
                            x.lang = None;
                        }
                    }
                    normalized::InlineBlock::Uncertain(x) => {
                        if x.lang == line_lang {
                            x.lang = None;
                        }
                    }
                    normalized::InlineBlock::Abbreviation(x) => {
                        if x.lang == line_lang {
                            x.lang = None;
                        }
                    }
                    normalized::InlineBlock::Correction(x) => {
                        let cor_lang = x.lang.clone();
                        if x.lang == line_lang {
                            x.lang = None;
                        }
                        // corrections also have lang on each of their versions
                        for version in &mut x.versions {
                            if version.lang == cor_lang {
                                version.lang = None;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(most_common_lang)
}

impl From<streamed::Correction> for normalized::Correction {
    fn from(value: streamed::Correction) -> Self {
        normalized::Correction {
            lang: value.lang().map(std::string::ToString::to_string),
            versions: value
                .versions
                .into_iter()
                .map(|v| normalized::Version {
                    lang: Some(v.lang),
                    hand: v.hand,
                    content: v.content,
                })
                .collect(),
        }
    }
}
impl From<streamed::Abbreviation> for normalized::Abbreviation {
    fn from(value: streamed::Abbreviation) -> Self {
        // the language of the entire abbreviation is the one used in the expansion
        // if the surface has another language, it is specifically set here
        normalized::Abbreviation {
            surface: crate::schema::AbbrSurface {
                lang: if value.expansion_lang == value.surface_lang {
                    None
                } else {
                    Some(value.surface_lang)
                },
                content: value.surface,
            },
            expansion: crate::schema::AbbrExpansion {
                lang: None,
                content: value.expansion,
            },
            lang: Some(value.expansion_lang),
        }
    }
}

impl From<streamed::Uncertain> for normalized::Uncertain {
    fn from(value: streamed::Uncertain) -> Self {
        normalized::Uncertain {
            lang: Some(value.lang),
            cert: value.cert,
            agent: value.agent,
            content: value.content,
        }
    }
}

impl From<streamed::Paragraph> for normalized::Paragraph {
    fn from(value: streamed::Paragraph) -> Self {
        normalized::Paragraph {
            lang: Some(value.lang),
            content: value.content,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::normalized;
    use crate::normalized::Abbreviation;
    use crate::streamed;

    /// We should be able to stream a normalized text
    #[test]
    fn stream_lines_inconsistent() {
        let xml = include_str!("../examples/01_all_elements.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        assert_eq!(
            streamed_res.unwrap_err(),
            super::StreamError::LineIndexInconsistent(1, 2)
        );
    }

    /// We should be able to stream a normalized text
    #[test]
    fn can_stream() {
        let xml = include_str!("../examples/02_lines_consistent.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        assert!(streamed_res.is_ok());
        let expected = streamed::Manuscript {
            meta: streamed::Meta {
                name: "Der Name voms dem Manuskripts".to_string(),
                page_nr: "34 verso".to_string(),
                title: "Manuskript Name folio 34 verso.".to_string(),
                institution: Some("University of does-not-exist".to_string()),
                collection: Some("Collectors Edition 2 electric boogaloo".to_string()),
                hand_desc: "There are two recognizable Hands: hand1 and hand2.".to_string(),
                script_desc: "Die Schrift in diesem Manuskript gibt es.".to_string(),
            },
            content: vec![
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "line 1 content. This line is completely preserved.".to_string(),
                }),
                streamed::Block::Lacuna(streamed::Lacuna {
                    reason: "lost".to_string(),
                    unit: streamed::ExtentUnit::Line,
                    n: 1,
                    cert: Some("high".to_string()),
                }),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content:
                        "Line 3 content - line2 is the line that was skipped by the previous gap"
                            .to_string(),
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr-x-babli".to_string(),
                    content: "Some stuff with babylonian Niqud".to_string(),
                }),
                streamed::Block::Break(streamed::BreakType::Column),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "Hier ein an".to_string(),
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Lacuna(streamed::Lacuna {
                    reason: "lost".to_string(),
                    unit: streamed::ExtentUnit::Column,
                    n: 2,
                    cert: Some("high".to_string()),
                }),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "text".to_string(),
                }),
            ],
        };
        assert_eq!(streamed_res.unwrap(), expected);
    }

    /// Taking a streamed text, destreaming and then restreaming it should be the identity
    ///
    /// NOTE:
    /// - for this identity to hold, the text has to have undergone language normalization.
    ///   otherwise, the languages may be correct, but set at different levels
    #[test]
    fn stream_circ_destream_is_identity() {
        let xml = include_str!("../examples/02_lines_consistent.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        assert!(streamed_res.is_ok());
        let streamed = streamed_res.unwrap();

        let destreamed: Result<normalized::Manuscript, _> = streamed.clone().try_into();
        assert!(destreamed.is_ok());
        let restreamed: Result<streamed::Manuscript, _> = destreamed.unwrap().try_into();
        assert!(restreamed.is_ok());
        assert_eq!(streamed, restreamed.unwrap());
    }

    /// Test that the language-normalization works correctly
    #[test]
    fn destream_language() {
        let xml = include_str!("../examples/03_language_normalization.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        assert!(streamed_res.is_ok());
        let streamed = streamed_res.unwrap();

        let destreamed: Result<normalized::Manuscript, _> = streamed.clone().try_into();
        assert!(destreamed.is_ok());
        let restreamed: Result<streamed::Manuscript, _> = destreamed.unwrap().try_into();
        assert!(restreamed.is_ok());
        assert_eq!(streamed, restreamed.unwrap());
    }

    #[test]
    fn stream_language_1() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: None,
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                        lang: None,
                        content: "text in hbo-Hebr".to_string(),
                    })],
                }],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![streamed::Block::Text(streamed::Paragraph {
                lang: "hbo-Hebr".to_string(),
                content: "text in hbo-Hebr".to_string()
            }),]
        );
    }

    #[test]
    fn stream_language_2() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: Some("grc".to_string()),
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                        lang: None,
                        content: "text".to_string(),
                    })],
                }],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![streamed::Block::Text(streamed::Paragraph {
                lang: "grc".to_string(),
                content: "text".to_string()
            }),]
        );
    }

    #[test]
    fn stream_language_3() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![
                    normalized::Line {
                        lang: Some("grc".to_string()),
                        n: 1,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                    normalized::Line {
                        lang: Some("grc".to_string()),
                        n: 2,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                    normalized::Line {
                        lang: None,
                        n: 3,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                ],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "text".to_string()
                }),
            ]
        );
        let destreamed: normalized::Text = streamed.try_into().unwrap();
        let expected = normalized::Text {
            lang: "grc".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![
                    normalized::Line {
                        lang: None,
                        n: 1,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                    normalized::Line {
                        lang: None,
                        n: 2,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                    normalized::Line {
                        lang: Some("hbo-Hebr".to_string()),
                        n: 3,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    },
                ],
            }],
        };
        assert_eq!(expected, destreamed);
    }

    #[test]
    fn stream_language_4() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![
                normalized::Column {
                    lang: Some("grc".to_string()),
                    n: 1,
                    lines: vec![
                        normalized::Line {
                            lang: None,
                            n: 1,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                        normalized::Line {
                            lang: Some("hbo-Hebr".to_string()),
                            n: 2,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                        normalized::Line {
                            lang: None,
                            n: 3,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                    ],
                },
                normalized::Column {
                    lang: Some("hbo-Hebr".to_string()),
                    n: 2,
                    lines: vec![normalized::Line {
                        lang: None,
                        n: 1,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: Some("grc".to_string()),
                            content: "text".to_string(),
                        })],
                    }],
                },
            ],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "text".to_string()
                }),
                streamed::Block::Break(streamed::BreakType::Line),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
                streamed::Block::Break(streamed::BreakType::Column),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
            ]
        );
        let destreamed: normalized::Text = streamed.try_into().unwrap();
        let expected = normalized::Text {
            lang: "grc".to_string(),
            columns: vec![
                normalized::Column {
                    lang: None,
                    n: 1,
                    lines: vec![
                        normalized::Line {
                            lang: None,
                            n: 1,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                        normalized::Line {
                            lang: Some("hbo-Hebr".to_string()),
                            n: 2,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                        normalized::Line {
                            lang: None,
                            n: 3,
                            blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                                lang: None,
                                content: "text".to_string(),
                            })],
                        },
                    ],
                },
                normalized::Column {
                    lang: None,
                    n: 2,
                    lines: vec![normalized::Line {
                        lang: None,
                        n: 1,
                        blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                            lang: None,
                            content: "text".to_string(),
                        })],
                    }],
                },
            ],
        };
        assert_eq!(expected, destreamed);
    }

    /// When streaming, there should be no final line and column breaks
    #[test]
    fn stream_final_breaks() {
        let normalized: normalized::Text = normalized::Text {
            lang: "lang".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: None,
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Text(normalized::Paragraph {
                        lang: None,
                        content: "content".to_string(),
                    })],
                }],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![streamed::Block::Text(streamed::Paragraph {
            lang: "lang".to_string(),
            content: "content".to_string(),
        })];
        assert_eq!(streamed, expected);
    }

    /// Language should get normalized while streaming/destreaming
    #[test]
    fn choice_language() {
        let normalized: normalized::Text = normalized::Text {
            lang: "lang".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: None,
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Abbreviation(Abbreviation {
                        lang: Some("IRRELEVANT".to_string()),
                        surface: crate::schema::AbbrSurface {
                            lang: Some("grc".to_string()),
                            content: "πιπι".to_string(),
                        },
                        expansion: crate::schema::AbbrExpansion {
                            lang: Some("hbo-Hebr".to_string()),
                            content: "יהוה".to_string(),
                        },
                    })],
                }],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![streamed::Block::Abbreviation(streamed::Abbreviation {
            surface_lang: "grc".to_string(),
            surface: "πιπι".to_string(),
            expansion_lang: "hbo-Hebr".to_string(),
            expansion: "יהוה".to_string(),
        })];
        assert_eq!(streamed, expected);

        let destreamed: normalized::Text = streamed.try_into().unwrap();
        let expected_destreamed: normalized::Text = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: None,
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Abbreviation(Abbreviation {
                        lang: None,
                        surface: crate::schema::AbbrSurface {
                            lang: Some("grc".to_string()),
                            content: "πιπι".to_string(),
                        },
                        expansion: crate::schema::AbbrExpansion {
                            lang: None,
                            content: "יהוה".to_string(),
                        },
                    })],
                }],
            }],
        };
        assert_eq!(destreamed, expected_destreamed);
    }

    #[test]
    fn correction_language() {
        let normalized: normalized::Text = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            columns: vec![normalized::Column {
                lang: None,
                n: 1,
                lines: vec![normalized::Line {
                    lang: None,
                    n: 1,
                    blocks: vec![normalized::InlineBlock::Correction(
                        normalized::Correction {
                            lang: None,
                            versions: vec![
                                normalized::Version {
                                    lang: None,
                                    hand: None,
                                    content: "יהוה".to_string(),
                                },
                                normalized::Version {
                                    lang: Some("hbo-Phnx".to_string()),
                                    hand: None,
                                    content: "𐤉𐤄𐤅𐤄".to_string(),
                                },
                            ],
                        },
                    )],
                }],
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![streamed::Block::Correction(streamed::Correction {
            versions: vec![
                streamed::Version {
                    lang: "hbo-Hebr".to_string(),
                    hand: None,
                    content: "יהוה".to_string(),
                },
                streamed::Version {
                    lang: "hbo-Phnx".to_string(),
                    hand: None,
                    content: "𐤉𐤄𐤅𐤄".to_string(),
                },
            ],
        })];
        assert_eq!(streamed, expected);
    }
}
