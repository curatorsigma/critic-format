//! The surface form of a text.
//!
//! This form essentially "forgets" all marked up structure and returns raw strings with the
//! ability to index between the surface form and the marked up form.

use crate::streamed::Block;

/// Split a Char at whitespaces and return the char indices at which words begin
pub struct SplitWhitespaceIndices<'a> {
    /// the remaining str
    remaining_str: &'a str,
    current_char_index: usize,
}
impl<'a> SplitWhitespaceIndices<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            remaining_str: input,
            current_char_index: 0,
        }
    }
}
impl Iterator for SplitWhitespaceIndices<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        // forward all whitespaces
        let mut split_off_bytes = 0;
        for char in self.remaining_str.chars().take_while(|c| c.is_whitespace()) {
            split_off_bytes += char.len_utf8();
            self.current_char_index += 1;
        }
        self.remaining_str = &self.remaining_str[split_off_bytes..];
        if self.remaining_str.is_empty() {
            return None;
        }
        let res = self.current_char_index;

        // forward all non-whitespaces
        let mut split_off_bytes = 0;
        for char in self
            .remaining_str
            .chars()
            .take_while(|c| !c.is_whitespace())
        {
            split_off_bytes += char.len_utf8();
            self.current_char_index += 1;
        }
        self.remaining_str = &self.remaining_str[split_off_bytes..];

        Some(res)
    }
}

#[test]
fn whitespace_indices() {
    let input = "Hi I am a string with whitespaces.";
    let whitespace_indices = SplitWhitespaceIndices::new(input).collect::<Vec<_>>();
    assert_eq!(whitespace_indices, vec![0, 3, 5, 8, 10, 17, 22]);
}

#[test]
fn whitespace_indices_whitespace_end() {
    let input = "Hi I am a string with whitespaces-at-the-ends.   ";
    let whitespace_indices = SplitWhitespaceIndices::new(input).collect::<Vec<_>>();
    assert_eq!(whitespace_indices, vec![0, 3, 5, 8, 10, 17, 22]);
}

#[test]
fn whitespace_indices_whitespace_start() {
    let input = "   \t Hi I am a string with whitespaces-at-the-start.";
    let whitespace_indices = SplitWhitespaceIndices::new(input).collect::<Vec<_>>();
    assert_eq!(whitespace_indices, vec![5, 8, 10, 13, 15, 22, 27]);
}

/// The position returned is the character index, not a byte position
#[test]
fn whitespace_indices_long_scalar() {
    let input = "word1\u{10ffff} word2";
    let whitespace_indices = SplitWhitespaceIndices::new(input).collect::<Vec<_>>();
    assert_eq!(whitespace_indices, vec![0, 7]);
}

/// Index between a position in the cleansed text and the marked up basetext.
#[derive(Debug)]
pub struct SurfaceIndex {
    /// The char position in the raw text where this index is positioned.
    position_in_raw: usize,
    /// The block of the input where this index is positioned.
    block_position: usize,
    /// The index in the block where this index is positioned.
    position_in_block: usize,
}
impl SurfaceIndex {
    #[must_use]
    pub fn position_in_raw(&self) -> usize {
        self.position_in_raw
    }

    #[must_use]
    pub fn block_position(&self) -> usize {
        self.block_position
    }

    #[must_use]
    pub fn position_in_block(&self) -> usize {
        self.position_in_block
    }
}

/// A portion of cleansed base text with the index between this cleansed form and the complete
/// marked up form.
#[derive(Debug, Default)]
pub struct SurfaceBaseText {
    /// The raw surface text
    raw_text: String,
    /// Indices mapping from the blocks to the raw text, one for each word (separated by normal rust
    /// whitespaces)
    indexmap: Vec<SurfaceIndex>,
}
impl core::fmt::Display for SurfaceBaseText {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.raw_text)
    }
}
impl SurfaceBaseText {
    #[must_use]
    pub fn raw_text(&self) -> &str {
        &self.raw_text
    }
    #[must_use]
    pub fn indexmap(&self) -> &[SurfaceIndex] {
        &self.indexmap
    }
    #[must_use]
    pub fn destructure(self) -> (String, Vec<SurfaceIndex>) {
        (self.raw_text, self.indexmap)
    }

    /// Take the chunk of basetext and extract the surface text.
    /// Characters not in `equality_alphabet` are ignored
    ///
    /// For Corrections, only the last version will be considered the base text.
    ///
    /// ## Example
    ///
    /// ```
    /// use critic_format::streamed::{Block, BreakType, Paragraph};
    /// use critic_format::surface_form::SurfaceBaseText;
    ///
    /// let streamed = vec![
    ///     Block::Break(BreakType::Page("page1".to_string())),
    ///     Block::Text(Paragraph {
    ///         lang: "de".to_string(),
    ///         content: "Und also sprach Zarathustra:".to_string()
    ///     }),
    ///     Block::Break(BreakType::Line),
    ///     Block::Text(Paragraph {
    ///         lang: "hbo".to_string(),
    ///         content: "ויאמר".to_string()
    ///     }),
    /// ];
    ///
    /// let surface_text = SurfaceBaseText::from_blocks_with_equality_alphabet(&streamed,
    /// Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"));
    ///
    /// let raw_text = "Und also sprach Zarathustra".to_string();
    /// assert_eq!(raw_text, surface_text.destructure().0);
    /// ```
    #[must_use]
    pub fn from_blocks_with_equality_alphabet(
        text_chunk: &[Block],
        equality_alphabet: Option<&str>,
    ) -> Self {
        let mut res = Self::default();
        for (block_idx, block) in text_chunk.iter().enumerate() {
            res.push_block(block_idx, block, equality_alphabet);
        }
        res
    }

    /// Take the chunk of basetext and cleanse it:
    ///
    /// ```
    /// use critic_format::streamed::{Block, BreakType, Paragraph};
    /// use critic_format::surface_form::SurfaceBaseText;
    ///
    /// let streamed = vec![
    ///     Block::Break(BreakType::Page("page1".to_string())),
    ///     Block::Text(Paragraph {
    ///         lang: "grc".to_string(),
    ///         content: "και ειπεν αυτου".to_string()
    ///     }),
    ///     Block::Break(BreakType::Line),
    ///     Block::Text(Paragraph {
    ///         lang: "grc".to_string(),
    ///         content: "και λεγει".to_string()
    ///     }),
    ///     Block::Break(BreakType::Line),
    ///     Block::Text(Paragraph {
    ///         lang: "hbo".to_string(),
    ///         content: "ויאמר".to_string()
    ///     }),
    /// ];
    ///
    /// let surface_text = SurfaceBaseText::from_blocks(&streamed);
    ///
    /// let raw_text = "και ειπεν αυτου και λεγει ויאמר".to_string();
    /// assert_eq!(raw_text, surface_text.destructure().0);
    /// ```
    #[must_use]
    pub fn from_blocks(text_chunk: &[Block]) -> Self {
        Self::from_blocks_with_equality_alphabet(text_chunk, None)
    }

    fn push_content(&mut self, block_idx: usize, content: &str, equality_alphabet: Option<&str>) {
        let equality_cleansed_paragraph = if let Some(ea) = equality_alphabet {
            content
                .chars()
                .filter(|c| ea.contains(*c) || c.is_whitespace())
                .collect::<String>()
        } else {
            content.to_string()
        };
        for (word_idx, word_position) in
            SplitWhitespaceIndices::new(&equality_cleansed_paragraph).enumerate()
        {
            self.indexmap.push(SurfaceIndex {
                position_in_raw: self.raw_text.len() + word_position,
                block_position: block_idx,
                position_in_block: word_idx,
            });
        }
        if !self.raw_text.is_empty() && !equality_cleansed_paragraph.is_empty() {
            self.raw_text.push(' ');
        }
        self.raw_text.push_str(&equality_cleansed_paragraph);
    }

    /// Push a block of content into this [`SurfaceBaseText`].
    fn push_block(&mut self, block_idx: usize, block: &Block, equality_alphabet: Option<&str>) {
        match block {
            Block::Text(paragraph) => {
                self.push_content(block_idx, &paragraph.content, equality_alphabet);
            }
            Block::Uncertain(uncertain) => {
                self.push_content(block_idx, &uncertain.content, equality_alphabet);
            }
            Block::Correction(correction) => {
                // use only the last version: this will be topmost on the page
                if let Some(last_version_content) = correction.versions.last() {
                    self.push_content(block_idx, &last_version_content.content, equality_alphabet);
                }
            }
            Block::Abbreviation(abbreviation) => {
                self.push_content(block_idx, &abbreviation.surface, equality_alphabet);
            }
            Block::Anchor(_) | Block::Break(_) | Block::Lacuna(_) | Block::Space(_) => {}
        }
    }
}
