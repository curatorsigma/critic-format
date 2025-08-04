//! Converting from [`normalized`] to [`streamed`] representation.
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
    /// The name of the first page must be given as a leading [`pagebreak`](streamed::BreakType::Page) in the stream of blocks.
    FirstPageNameMissing,
    /// There was a column without lines in it.
    ///
    /// This needs to be marked as a column spanning lacuna instead
    NoLinesInColumn(i32),
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
            Self::FirstPageNameMissing => {
                write!(
                    f,
                    "The name of the first page must be given in a leading PageBreak"
                )
            }
            Self::NoLinesInColumn(col_idx) => {
                write!(
                    f,
                    "Column {col_idx} contained no lines. Please mark this as a column-spanning lacuna instead."
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

/// An iterator over the individual [`Block`](streamed::Block)s representing a single
/// [`Page`](normalized::Page).
struct BlocksFromPage<'a> {
    remaining_cols_on_page: std::vec::IntoIter<normalized::Column>,
    remaining_lines_in_col: std::vec::IntoIter<normalized::Line>,
    remaining_blocks_in_line: std::vec::IntoIter<normalized::InlineBlock>,

    /// the language of the current line
    /// This gets updated while advancing through the manuscript, taking the value of the current
    /// column/line
    current_language: String,
    /// the default language in this MS
    /// This is static and will not be updated.
    default_language: std::borrow::Cow<'a, str>,
    /// the default language in the current column
    language_in_col: String,
    /// the logical numbering of the column (i.e. getting larger when passing column-spanning
    /// lacuna)
    col_idx: i32,
    /// the logical numbering of the line (i.e. getting larger when passing line-spanning
    /// lacuna)
    line_idx: i32,
    /// Will be initialized as `Some(own name)`.
    ///
    /// When `Some(x)`, will output `PageBreak(x)`, taking ownership and leaving None here
    return_own_startbreak_next: Option<String>,
    /// signals that the next `Break(BreakType::Line)` should be skipped
    skip_next_linebreak: bool,
    /// signals that the next `Break(BreakType::Column)` should be skipped
    skip_next_columnbreak: bool,
}
impl<'a> BlocksFromPage<'a> {
    pub fn new(page: normalized::Page, default_language: &'a str) -> Self {
        // if defined on the page, use that
        // else, use `default_language`
        let default_language = if let Some(l) = page.lang.clone() {
            std::borrow::Cow::Owned(l)
        } else {
            std::borrow::Cow::Borrowed(default_language)
        };
        let col_iter = page.columns.into_iter();
        Self {
            remaining_cols_on_page: col_iter,
            remaining_lines_in_col: vec![].into_iter(),
            remaining_blocks_in_line: vec![].into_iter(),
            current_language: default_language.to_string(),
            language_in_col: default_language.to_string(),
            default_language,
            col_idx: 0,
            line_idx: 0,
            return_own_startbreak_next: Some(page.n),
            skip_next_linebreak: false,
            // the column break after the initial page break has to be skipped
            skip_next_columnbreak: true,
        }
    }

    /// Advance the internal state by one line, taking the data from `next_line` into the block
    /// iterator and updating language.
    ///
    /// This also checks line index consistency and errors if the line index is inconsistent
    fn load_next_line(&mut self, next_line: normalized::Line) -> Result<(), StreamError> {
        // a new logical line has started - increase the logical line number
        self.line_idx += 1;
        // this lines language is either given, or supplied from the column
        if let Some(new_lang) = next_line.lang {
            self.current_language.clone_from(&new_lang);
        } else {
            self.current_language = self.language_in_col.clone();
        }
        if self.line_idx != next_line.n {
            return Err(StreamError::LineIndexInconsistent(
                self.line_idx,
                next_line.n,
            ));
        }
        // these are the blocks on the new line
        self.remaining_blocks_in_line = next_line.blocks.into_iter();
        Ok(())
    }

    /// Advance the internal state by one column, taking the data from `next_column` into the block
    /// iterator and update language.
    ///
    /// This also checks column index consistency and errors if the line index is inconsistent
    ///
    /// We immediately load the first line for this column as well.
    ///
    /// Returns:
    /// - true IFF this was the first column loaded
    fn load_next_column(&mut self, next_column: normalized::Column) -> Result<(), StreamError> {
        self.col_idx += 1;
        self.line_idx = 0;
        // this columns language is either given, or supplied from the page
        self.language_in_col = if let Some(new_lang) = next_column.lang {
            new_lang.clone()
        } else {
            self.default_language.to_string()
        };
        self.current_language = self.language_in_col.clone();

        if self.col_idx != next_column.n {
            return Err(StreamError::ColumnIndexInconsistent(
                self.col_idx,
                next_column.n,
            ));
        }
        // get the lines for the next column into our internal iterator
        self.remaining_lines_in_col = next_column.lines.into_iter();
        // now get the blocks for the first line into their iterator
        match self.remaining_lines_in_col.next() {
            Some(next_line) => {
                self.load_next_line(next_line)?;
            }
            None => {
                return Err(StreamError::NoLinesInColumn(self.col_idx));
            }
        }
        Ok(())
    }
}
impl Iterator for BlocksFromPage<'_> {
    type Item = Result<streamed::Block, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(own_name) = self.return_own_startbreak_next.take() {
            return Some(Ok(streamed::Block::Break(streamed::BreakType::Page(
                own_name,
            ))));
        }

        // get the next block, or handle the case where the line/column has ended
        let Some(curr_block) = self.remaining_blocks_in_line.next() else {
            // this line is exhausted, go to the next one
            match self.remaining_lines_in_col.next() {
                Some(next_line) => {
                    if let Err(e) = self.load_next_line(next_line) {
                        return Some(Err(e));
                    }
                    // but for now, we need to return the line break - the next iteration will get
                    // the first block of the new line
                    if self.skip_next_linebreak {
                        self.skip_next_linebreak = false;
                        return self.next();
                    }
                    return Some(Ok(streamed::Block::Break(streamed::BreakType::Line)));
                }
                // this column in its entirety is exhausted, go to the next
                None => {
                    match self.remaining_cols_on_page.next() {
                        Some(next_column) => {
                            if let Err(e) = self.load_next_column(next_column) {
                                return Some(Err(e));
                            }
                            if self.skip_next_columnbreak {
                                self.skip_next_columnbreak = false;
                                return self.next();
                            }
                            return Some(Ok(streamed::Block::Break(streamed::BreakType::Column)));
                        }
                        // there are no more columns, this page is done
                        // Do not add final Line and Column breaks here
                        None => {
                            return None;
                        }
                    };
                }
            };
        };

        let block_lang = curr_block.language().map_or_else(
            || self.current_language.clone(),
            std::string::ToString::to_string,
        );
        let streamed_block = match (block_lang, curr_block).try_into() {
            Ok(x) => x,
            Err(e) => {
                return Some(Err(e));
            }
        };
        // break off if we start a multi-line or multi-column gap with this block
        match streamed_block {
            // a lacuna spanning multiple lines.
            // we increment the line-nr by the appropriate amount and continue
            // streaming from the next line defined in the xml
            //
            // There are n lines skipped, so the next defined line has index +n (from
            // skipped lines) relative to the current line
            // the `<gap>` is on.
            //
            // (the other lines are NOT to be defined in the xml, since they are taken
            // up by the `<gap>`)
            streamed::Block::Lacuna(streamed::Lacuna {
                unit: normalized::ExtentUnit::Line,
                n,
                ..
            }) => {
                self.line_idx += n;
                self.skip_next_linebreak = true;
                return Some(Ok(streamed_block));
            }
            // a lacuna spanning multiple columns.
            // we increment the column-nr by the appropriate amount and continue
            // streaming from the next column defined in the xml
            streamed::Block::Lacuna(streamed::Lacuna {
                unit: normalized::ExtentUnit::Column,
                n,
                ..
            }) => {
                self.col_idx += n;
                self.skip_next_columnbreak = true;
                return Some(Ok(streamed_block));
            }
            // now we do the exact same thing for spaces
            streamed::Block::Space(streamed::Space {
                unit: normalized::ExtentUnit::Line,
                quantity,
                ..
            }) => {
                self.line_idx += quantity;
                self.skip_next_linebreak = true;
                return Some(Ok(streamed_block));
            }
            // a lacuna spanning multiple columns.
            // we increment the column-nr by the appropriate amount and continue
            // streaming from the next column defined in the xml
            streamed::Block::Space(streamed::Space {
                unit: normalized::ExtentUnit::Column,
                quantity,
                ..
            }) => {
                self.col_idx += quantity;
                self.skip_next_columnbreak = true;
                return Some(Ok(streamed_block));
            }
            _ => {}
        }
        Some(Ok(streamed_block))
    }
}

impl normalized::Page {
    fn into_streamed(self, default_language: &str) -> BlocksFromPage {
        BlocksFromPage::new(self, default_language)
    }
}

impl TryFrom<normalized::Text> for Vec<streamed::Block> {
    type Error = StreamError;

    fn try_from(value: normalized::Text) -> Result<Self, Self::Error> {
        let streamed_blocks = value
            .pages
            .into_iter()
            .flat_map(|p| p.into_streamed(&value.lang));
        streamed_blocks.collect::<Result<Vec<_>, _>>()
    }
}

/*
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
                        // now we do the exact same thing for spaces
                        streamed::Block::Space(streamed::Space {
                            unit: normalized::ExtentUnit::Line,
                            quantity,
                            ..
                        }) => {
                            line_idx += quantity + 1;
                            res.push(streamed_block);
                            continue 'line;
                        }
                        // a lacuna spanning multiple columns.
                        // we increment the column-nr by the appropriate amount and continue
                        // streaming from the next column defined in the xml
                        streamed::Block::Space(streamed::Space {
                            unit: normalized::ExtentUnit::Column,
                            quantity,
                            ..
                        }) => {
                            col_idx += quantity + 1;
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
*/

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
            normalized::InlineBlock::Space(x) => streamed::Block::Space(x),
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

/// consume a stream of blocks until the current page ends
///
/// then return the built page and the name of the NEXT [`Page`]
/// - this is None, if the stream simply ended without us knowing the name of the next page
/// - this name is part of the [`BreakType`](streamed::BreakType) ending this [`Page`], which we have to consume to see it
///
/// early return on any error; the stream will be in an undefined state when this fn errs.
/// You may forward to the next [`BreakType::Page`](streamed::BreakType::Page), consume it and then continue with the next page if
/// you want to unroll
///
/// [`Page`]: normalized::Page
fn transform_until_page_end(
    stream: &mut impl Iterator<Item = streamed::Block>,
    page_nr: String,
) -> Result<(normalized::Page, Option<String>), StreamError> {
    // these are logical indices we are building, keeping track of lacuna sizes
    let mut line_idx = 1;
    let mut column_idx = 1;
    // these dicts contain info about the language used throughout the MS
    let mut language_use = HashMap::<String, i32>::new();
    let mut language_use_in_col = HashMap::<String, i32>::new();
    let mut language_use_in_line = HashMap::<String, i32>::new();
    // these are the vecs we are trying to fill with this function
    let mut columns = Vec::<normalized::Column>::new();
    let mut lines = Vec::<normalized::Line>::new();
    let mut blocks_in_line = Vec::<normalized::InlineBlock>::new();

    let next_page_name = 'stream: loop {
        let Some(block) = stream.next() else {
            // the stream has ended - this is the last page in this stream
            break 'stream None;
        };
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

        // this page is done, and there is another one afterwards
        if let streamed::Block::Break(streamed::BreakType::Page(next_name)) = block {
            break 'stream Some(next_name);
        }
        // add this block to this line:
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
    };

    // now we need to add the remaining blocks as a final line/column as in a column break
    if !blocks_in_line.is_empty() {
        // first end the line
        let most_common_lang_in_line =
            most_common_lang(&language_use_in_line).map(std::string::ToString::to_string);
        lines.push(normalized::Line {
            lang: most_common_lang_in_line,
            n: line_idx,
            blocks: core::mem::take(&mut blocks_in_line),
        });
    }
    if !lines.is_empty() {
        // now end the column and go to the next one
        let most_common_lang_in_col =
            most_common_lang(&language_use_in_col).map(std::string::ToString::to_string);
        let take_lines = core::mem::take(&mut lines);
        columns.push(normalized::Column {
            lang: most_common_lang_in_col,
            n: column_idx,
            lines: take_lines,
        });
    }

    let most_common_lang = normalize_language(&mut columns, &language_use)?;

    Ok((
        normalized::Page {
            lang: Some(most_common_lang.to_string()),
            columns,
            n: page_nr,
        },
        next_page_name,
    ))
}

impl TryFrom<Vec<streamed::Block>> for normalized::Text {
    type Error = StreamError;

    fn try_from(value: Vec<streamed::Block>) -> Result<Self, Self::Error> {
        let mut blocks_iter = value.into_iter();
        let mut pages = Vec::new();
        let mut langs_in_text = HashMap::<String, i32>::new();
        let mut next_page_name = match blocks_iter.next() {
            None => {
                // empty text, just return a trivial Text
                most_common_lang(&langs_in_text);
                return Ok(normalized::Text {
                    lang: String::default(),
                    pages,
                });
            }
            Some(streamed::Block::Break(streamed::BreakType::Page(x))) => x,
            Some(_) => {
                return Err(StreamError::FirstPageNameMissing);
            }
        };
        loop {
            let (this_page, new_name) = transform_until_page_end(&mut blocks_iter, next_page_name)?;
            let most_common_lang_in_page = this_page
                .lang
                .as_ref()
                .expect("transform_until_page_end always sets language");
            if let Some(this_lang_val) = langs_in_text.get_mut(most_common_lang_in_page) {
                *this_lang_val += 1;
            } else {
                langs_in_text.insert(most_common_lang_in_page.to_string(), 1);
            }
            pages.push(this_page);
            let Some(y) = new_name else {
                // the stream has ended without producing a name for the next page
                // (i.e. there is no next page)
                // Now normalize Languages over the pages
                let most_common_lang = most_common_lang(&langs_in_text)
                    .unwrap_or_default()
                    .to_string();
                normalize_language_over_pages(&mut pages, &most_common_lang);
                return Ok(normalized::Text {
                    lang: most_common_lang,
                    pages,
                });
            };
            next_page_name = y;
        }
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

// this function is admittedly ugly - however, most of it is is one large match statement which
// does not refactor into meaningful functions
/// Add a block to this pages datastructure,
/// update language use and forward line and column indexes when
/// a line or column is ended by this block
///
/// # Panics
/// MUST NOT BE CALLED on `block = Block::Break(BreakType::Page())!`
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
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
        streamed::Block::Break(streamed::BreakType::Page(_n)) => {
            panic!(
                "handle_block MUST NOT be called on a pagebreak; \
                you need to handle this because handle_block cannot split pages"
            )
        }
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
        streamed::Block::Space(
            s @ streamed::Space {
                unit: streamed::ExtentUnit::Line,
                quantity: extent,
            },
        ) => {
            // first push the lacuna itself as a block
            blocks_in_line.push(normalized::InlineBlock::Space(s));
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
        streamed::Block::Space(
            s @ streamed::Space {
                unit: streamed::ExtentUnit::Column,
                quantity: extent,
                ..
            },
        ) => {
            // first push the space itself as a block
            blocks_in_line.push(normalized::InlineBlock::Space(s));
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
        streamed::Block::Lacuna(
            l @ streamed::Lacuna {
                unit: streamed::ExtentUnit::Character,
                ..
            },
        ) => {
            blocks_in_line.push(normalized::InlineBlock::Lacuna(l));
        }
        streamed::Block::Space(
            s @ streamed::Space {
                unit: streamed::ExtentUnit::Character,
                ..
            },
        ) => {
            blocks_in_line.push(normalized::InlineBlock::Space(s));
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
                    normalized::InlineBlock::Space(_)
                    | normalized::InlineBlock::Lacuna(_)
                    | normalized::InlineBlock::Anchor(_) => {}
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

/// Iterate over pages, remove language on pages where they are the same as the language of the
/// entire MS
fn normalize_language_over_pages(pages: &mut Vec<normalized::Page>, most_common_lang: &str) {
    for page in pages {
        if let Some(ref most_common_lang_in_page) = page.lang {
            if most_common_lang_in_page == most_common_lang {
                page.lang = None;
            }
        }
    }
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

    /// In this example text, a line index is inconsistent
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
        dbg!(&xml_res);
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        dbg!(&norm_res);
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        dbg!(&streamed_res);
        assert!(streamed_res.is_ok());
        let expected = streamed::Manuscript {
            meta: streamed::Meta {
                alt_identifier: vec![],
                title: "Manuskript Name".to_string(),
                institution: Some("University of does-not-exist".to_string()),
                collection: Some("Collectors Edition 2 electric boogaloo".to_string()),
                hand_desc: Some("There are two recognizable Hands: hand1 and hand2.".to_string()),
                script_desc: Some("Die Schrift in diesem Manuskript gibt es.".to_string()),
            },
            content: vec![
                streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
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

        let xml = include_str!("../examples/05_with_nontrivial_space.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        dbg!(&streamed_res);
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
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "hbo-Hebr".to_string(),
                    content: "text in hbo-Hebr".to_string()
                }),
            ]
        );
    }

    #[test]
    fn stream_language_2() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            pages: vec![normalized::Page {
                lang: Some("syr".to_string()),
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
                streamed::Block::Text(streamed::Paragraph {
                    lang: "grc".to_string(),
                    content: "text".to_string()
                }),
            ]
        );
    }

    #[test]
    fn stream_language_3() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
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
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        assert_eq!(expected, destreamed);
    }

    #[test]
    fn stream_language_4() {
        let normalized = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
                columns: vec![
                    normalized::Column {
                        lang: Some("grc".to_string()),
                        n: 1,
                        lines: vec![
                            normalized::Line {
                                lang: None,
                                n: 1,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
                            },
                            normalized::Line {
                                lang: Some("hbo-Hebr".to_string()),
                                n: 2,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
                            },
                            normalized::Line {
                                lang: None,
                                n: 3,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        assert_eq!(
            streamed,
            vec![
                streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
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
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
                columns: vec![
                    normalized::Column {
                        lang: None,
                        n: 1,
                        lines: vec![
                            normalized::Line {
                                lang: None,
                                n: 1,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
                            },
                            normalized::Line {
                                lang: Some("hbo-Hebr".to_string()),
                                n: 2,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
                            },
                            normalized::Line {
                                lang: None,
                                n: 3,
                                blocks: vec![normalized::InlineBlock::Text(
                                    normalized::Paragraph {
                                        lang: None,
                                        content: "text".to_string(),
                                    },
                                )],
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
            }],
        };
        assert_eq!(expected, destreamed);
    }

    /// When streaming, there should be no final line and column breaks
    #[test]
    fn stream_final_breaks() {
        let normalized: normalized::Text = normalized::Text {
            lang: "lang".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![
            streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
            streamed::Block::Text(streamed::Paragraph {
                lang: "lang".to_string(),
                content: "content".to_string(),
            }),
        ];
        assert_eq!(streamed, expected);
    }

    /// Language should get normalized while streaming/destreaming
    #[test]
    fn choice_language() {
        let normalized: normalized::Text = normalized::Text {
            lang: "lang".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![
            streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
            streamed::Block::Abbreviation(streamed::Abbreviation {
                surface_lang: "grc".to_string(),
                surface: "πιπι".to_string(),
                expansion_lang: "hbo-Hebr".to_string(),
                expansion: "יהוה".to_string(),
            }),
        ];
        assert_eq!(streamed, expected);

        let destreamed: normalized::Text = streamed.try_into().unwrap();
        let expected_destreamed: normalized::Text = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        assert_eq!(destreamed, expected_destreamed);
    }

    #[test]
    fn correction_language() {
        let normalized: normalized::Text = normalized::Text {
            lang: "hbo-Hebr".to_string(),
            pages: vec![normalized::Page {
                lang: None,
                n: "page1".to_string(),
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
            }],
        };
        let streamed: Vec<streamed::Block> = normalized.try_into().unwrap();
        let expected = vec![
            streamed::Block::Break(streamed::BreakType::Page("page1".to_string())),
            streamed::Block::Correction(streamed::Correction {
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
            }),
        ];
        assert_eq!(streamed, expected);
    }

    /// Multiple pages with different language that needs to be normalized and some gaps
    #[test]
    fn multi_page() {
        let xml = include_str!("../examples/07_multi-page.xml");
        let xml_res: Result<crate::schema::Tei, _> = quick_xml::de::from_str(xml);
        dbg!(&xml_res);
        assert!(xml_res.is_ok());
        let norm_res: Result<crate::normalized::Manuscript, _> = xml_res.unwrap().try_into();
        assert!(norm_res.is_ok());
        let streamed_res: Result<streamed::Manuscript, _> = norm_res.unwrap().try_into();
        dbg!(&streamed_res);
        assert!(streamed_res.is_ok());
        let streamed = streamed_res.unwrap();

        let destreamed: Result<normalized::Manuscript, _> = streamed.clone().try_into();
        assert!(destreamed.is_ok());
        let restreamed: Result<streamed::Manuscript, _> = destreamed.unwrap().try_into();
        assert!(restreamed.is_ok());
        assert_eq!(streamed, restreamed.unwrap());
    }
}
