//! The surface form of a text.
//!
//! This form essentially "forgets" all marked up structure and returns raw strings with the
//! ability to index between the surface form and the marked up form.

use crate::streamed::Block;

fn addr_of(s: &str) -> usize {
    s.as_ptr() as usize
}

/// Get the positions of word starts in `s`.
fn split_whitespace_indices(s: &str) -> impl Iterator<Item = usize> + use<'_> {
    s.split_whitespace()
        .map(move |sub| (addr_of(sub) - addr_of(s)))
}

#[test]
fn whitespace_indices() {
    let input = "Hi I am a string with whitespaces.";
    let whitespace_indices = split_whitespace_indices(input).collect::<Vec<_>>();
    assert_eq!(whitespace_indices, vec![0, 3, 5, 8, 10, 17, 22]);
}

/// Index between a position in the cleansed text and the marked up basetext.
#[derive(Debug)]
pub struct SurfaceIndex {
    /// The byte index in the raw text where this index is positioned.
    position_in_raw: usize,
    /// The block of the input, where this index is positioned in.
    block_position: usize,
    /// The byte position in the content of the block this index is positioned in.
    position_in_block: usize,
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
    /// Characters not in `equality_alphabet` are ignored
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
            split_whitespace_indices(&equality_cleansed_paragraph).enumerate()
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
