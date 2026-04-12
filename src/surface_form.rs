//! The surface form of a text.
//!
//! This form essentially "forgets" all marked up structure and returns raw strings with the
//! ability to index between the surface form and the marked up form.

use crate::streamed::Block;

/// Split a &[`str`] at whitespaces and return the char indices at which words begin
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SurfaceIndex {
    /// The char position in the raw text where this index is positioned.
    position_in_raw: usize,
    /// The block of the input where this index is positioned.
    block_position: usize,
    /// The index in the block where this index is positioned.
    position_in_block: usize,
}
impl SurfaceIndex {
    /// The character position in the raw text where this [`SurfaceIndex`] is positioned.
    ///
    /// This is a character index, not a byte index.
    #[must_use]
    pub fn position_in_raw(&self) -> usize {
        self.position_in_raw
    }

    /// The block index where this [`SurfaceIndex`] is positioned.
    ///
    /// This is the index in the actual `[Vec<Block>]`.
    #[must_use]
    pub fn block_position(&self) -> usize {
        self.block_position
    }

    /// The index of the word where this [`SurfaceIndex`] is positioned.
    ///
    /// This is the 0-based index of the word inside the block this [`SurfaceIndex`] belongs to.
    #[must_use]
    pub fn position_in_block(&self) -> usize {
        self.position_in_block
    }
}

/// A portion of cleansed base text with the index between this cleansed form and the complete
/// marked up form.
#[derive(Debug, Default)]
pub struct SurfaceBaseText {
    /// The length of `raw_text` in chars to prevent recalculation
    current_total_char_length: usize,
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

    /// Push an uncleansed string of content into this [`SurfaceBaseText`] at the given `block_idx`
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
                position_in_raw: self.current_total_char_length + word_position,
                block_position: block_idx,
                position_in_block: word_idx,
            });
        }
        if !self.raw_text.is_empty() && !equality_cleansed_paragraph.is_empty() {
            self.raw_text.push(' ');
            self.current_total_char_length += 1;
        }
        self.raw_text.push_str(&equality_cleansed_paragraph);
        self.current_total_char_length += equality_cleansed_paragraph.chars().count();
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

#[cfg(test)]
mod test {
    use crate::{
        streamed::{Anchor, Block, Paragraph},
        surface_form::{SurfaceBaseText, SurfaceIndex},
    };

    #[test]
    fn complicated_content() {
        let content = vec![
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-1".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "מ\u{5b7}ש\u{5c2}\u{5bc}\u{5b8}\u{596}א ג\u{5bc}\u{5b5}\u{5a3}יא ח\u{5b4}ז\u{5bc}\u{5b8}י\u{591}ו\u{5b9}ן מ\u{5b7}ה־ל\u{5bc}\u{5b8}\u{5a3}ך\u{5b0} א\u{5b5}פ\u{594}ו\u{5b9}א כ\u{5bc}\u{5b4}י־ע\u{5b8}ל\u{5b4}\u{5a5}ית כ\u{5bc}\u{5bb}ל\u{5bc}\u{5b8}\u{596}ך\u{5b0} ל\u{5b7}ג\u{5bc}\u{5b7}ג\u{5bc}\u{5bd}ו\u{5b9}ת׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-2".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ת\u{5bc}\u{5b0}ש\u{5c1}\u{5bb}א\u{5a3}ו\u{5b9}ת ׀ מ\u{5b0}ל\u{5b5}א\u{5b8}\u{597}ה ע\u{5b4}\u{59a}יר ה\u{5bd}ו\u{5b9}מ\u{5b4}י\u{5bc}\u{5b8}\u{594}ה ק\u{5b4}ר\u{5b0}י\u{5b8}\u{596}ה ע\u{5b7}ל\u{5bc}\u{5b4}יז\u{5b8}\u{591}ה ח\u{5b2}ל\u{5b8}ל\u{5b7}\u{599}י\u{5b4}ך\u{5b0}\u{599} ל\u{5b9}\u{5a3}א ח\u{5b7}ל\u{5b0}ל\u{5b5}י־ח\u{5b6}\u{594}ר\u{5b6}ב ו\u{5b0}ל\u{5b9}\u{596}א מ\u{5b5}ת\u{5b5}\u{5a5}י מ\u{5b4}ל\u{5b0}ח\u{5b8}מ\u{5b8}\u{5bd}ה׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-3".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "כ\u{5bc}\u{5c7}ל־ק\u{5b0}צ\u{5b4}ינ\u{5b7}\u{5a5}י\u{5b4}ך\u{5b0} נ\u{5b8}\u{5bd}ד\u{5b0}\u{fb1e}דו\u{5bc}־י\u{5b7}\u{596}ח\u{5b7}ד מ\u{5b4}ק\u{5bc}\u{5b6}\u{5a3}ש\u{5c1}\u{5b6}ת א\u{5bb}ס\u{5bc}\u{5b8}\u{591}רו\u{5bc} כ\u{5bc}\u{5c7}ל־נ\u{5b4}מ\u{5b0}צ\u{5b8}א\u{5b7}\u{599}י\u{5b4}ך\u{5b0}\u{599} א\u{5bb}ס\u{5bc}\u{5b0}ר\u{5a3}ו\u{5bc} י\u{5b7}ח\u{5b0}ד\u{5bc}\u{5b8}\u{594}ו מ\u{5b5}ר\u{5b8}ח\u{596}ו\u{5b9}ק ב\u{5bc}\u{5b8}ר\u{5b8}\u{5bd}חו\u{5bc}׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-4".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ע\u{5b7}ל־כ\u{5bc}\u{5b5}\u{5a5}ן א\u{5b8}מ\u{5b7}\u{59b}ר\u{5b0}ת\u{5bc}\u{5b4}י ש\u{5c1}\u{5b0}ע\u{5a5}ו\u{5bc} מ\u{5b4}נ\u{5bc}\u{5b4}\u{596}י א\u{5b2}מ\u{5b8}ר\u{5b5}\u{5a3}ר ב\u{5bc}\u{5b7}ב\u{5bc}\u{5b6}\u{591}כ\u{5b4}י א\u{5b7}ל־ת\u{5bc}\u{5b8}א\u{5b4}\u{5a3}יצו\u{5bc} ל\u{5b0}נ\u{5b7}\u{5bd}ח\u{5b2}מ\u{5b5}\u{594}נ\u{5b4}י ע\u{5b7}ל־ש\u{5c1}\u{5b9}\u{596}ד ב\u{5bc}\u{5b7}ת־ע\u{5b7}מ\u{5bc}\u{5b4}\u{5bd}י׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-5".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "כ\u{5bc}\u{5b4}\u{5a3}י יו\u{5b9}ם\u{5a9} מ\u{5b0}הו\u{5bc}מ\u{5b8}\u{5a8}ה ו\u{5bc}מ\u{5b0}בו\u{5bc}ס\u{5b8}\u{59c}ה ו\u{5bc}מ\u{5b0}בו\u{5bc}כ\u{5b8}\u{597}ה ל\u{5b7}\u{5bd}אד\u{5b9}נ\u{5b8}\u{5a7}י י\u{5b1}ה\u{5b9}ו\u{5b4}\u{59b}ה צ\u{5b0}ב\u{5b8}א\u{596}ו\u{5b9}ת ב\u{5bc}\u{5b0}ג\u{5b5}\u{5a3}י ח\u{5b4}ז\u{5bc}\u{5b8}י\u{591}ו\u{5b9}ן מ\u{5b0}ק\u{5b7}ר\u{5b0}ק\u{5b7}\u{5a5}ר ק\u{5b4}\u{596}ר ו\u{5b0}ש\u{5c1}\u{5a5}ו\u{5b9}ע\u{5b7} א\u{5b6}ל־ה\u{5b8}ה\u{5b8}\u{5bd}ר׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-6".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5b0}ע\u{5b5}יל\u{5b8}ם\u{599} נ\u{5b8}ש\u{5c2}\u{5b8}\u{5a3}א א\u{5b7}ש\u{5c1}\u{5b0}פ\u{5bc}\u{5b8}\u{594}ה ב\u{5bc}\u{5b0}ר\u{5b6}\u{5a5}כ\u{5b6}ב א\u{5b8}ד\u{5b8}\u{596}ם פ\u{5bc}\u{5b8}ר\u{5b8}ש\u{5c1}\u{5b4}\u{591}ים ו\u{5b0}ק\u{5b4}\u{5a5}יר ע\u{5b5}ר\u{5b8}\u{596}ה מ\u{5b8}ג\u{5b5}\u{5bd}ן׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-7".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5b7}י\u{5b0}ה\u{5b4}\u{5a5}י מ\u{5b4}ב\u{5b0}ח\u{5b7}ר־ע\u{5b2}מ\u{5b8}ק\u{5b7}\u{596}י\u{5b4}ך\u{5b0} מ\u{5b8}\u{5a3}ל\u{5b0}או\u{5bc} ר\u{5b8}\u{591}כ\u{5b6}ב ו\u{5b0}ה\u{5b7}פ\u{5bc}\u{5b8}\u{5a3}ר\u{5b8}ש\u{5c1}\u{5b4}\u{594}ים ש\u{5c1}\u{5b9}\u{596}ת ש\u{5c1}\u{5b8}\u{5a5}תו\u{5bc} ה\u{5b7}ש\u{5c1}\u{5bc}\u{5b8}\u{5bd}ע\u{5b0}ר\u{5b8}ה׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-8".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5b7}י\u{5b0}ג\u{5b7}\u{595}ל א\u{5b5}\u{596}ת מ\u{5b8}ס\u{5b7}\u{5a3}ך\u{5b0} י\u{5b0}הו\u{5bc}ד\u{5b8}\u{591}ה ו\u{5b7}ת\u{5bc}\u{5b7}ב\u{5bc}\u{5b5}ט\u{599} ב\u{5bc}\u{5b7}י\u{5bc}\u{5a3}ו\u{5b9}ם ה\u{5b7}ה\u{594}ו\u{5bc}א א\u{5b6}ל־נ\u{5b6}\u{596}ש\u{5c1}\u{5b6}ק ב\u{5bc}\u{5b5}\u{5a5}ית ה\u{5b7}י\u{5bc}\u{5b8}\u{5bd}ע\u{5b7}ר׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-9".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5b0}א\u{5b5}\u{5a8}ת ב\u{5bc}\u{5b0}ק\u{5b4}יע\u{5b5}\u{5a7}י ע\u{5b4}יר־ד\u{5bc}\u{5b8}ו\u{5b4}\u{59b}ד ר\u{5b0}א\u{5b4}ית\u{5b6}\u{596}ם כ\u{5bc}\u{5b4}י־ר\u{5b8}\u{591}ב\u{5bc}ו\u{5bc} ו\u{5b7}\u{5bd}ת\u{5bc}\u{5b0}ק\u{5b7}ב\u{5bc}\u{5b0}צ\u{594}ו\u{5bc} א\u{5b6}ת־מ\u{5b5}\u{5a5}י ה\u{5b7}ב\u{5bc}\u{5b0}ר\u{5b5}כ\u{5b8}\u{596}ה ה\u{5b7}ת\u{5bc}\u{5b7}ח\u{5b0}ת\u{5bc}ו\u{5b9}נ\u{5b8}\u{5bd}ה׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-10".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5b0}א\u{5b6}ת־ב\u{5bc}\u{5b8}ת\u{5bc}\u{5b5}\u{5a5}י י\u{5b0}רו\u{5bc}ש\u{5c1}\u{5b8}ל\u{5b7}\u{596}\u{34f}\u{5b4}ם ס\u{5b0}פ\u{5b7}ר\u{5b0}ת\u{5bc}\u{5b6}\u{591}ם ו\u{5b7}ת\u{5bc}\u{5b4}ת\u{5b0}צו\u{5bc}\u{599} ה\u{5b7}ב\u{5bc}\u{5b8}\u{5a3}ת\u{5bc}\u{5b4}\u{594}ים ל\u{5b0}ב\u{5b7}צ\u{5bc}\u{5b5}\u{596}ר ה\u{5b7}חו\u{5b9}מ\u{5b8}\u{5bd}ה׃".to_string(),
        },
    ),
    Block::Anchor(
        Anchor {
            anchor_id: "A_V_MT_Isaiah-22-11".to_string(),
            anchor_type: "Masoretic".to_string(),
        },
    ),
    Block::Text(
        Paragraph {
            lang: "hbo".to_string(),
            content: "ו\u{5bc}מ\u{5b4}ק\u{5b0}ו\u{5b8}\u{5a3}ה ׀ ע\u{5b2}ש\u{5c2}\u{5b4}ית\u{5b6}\u{597}ם ב\u{5bc}\u{5b5}\u{59a}ין ה\u{5b7}ח\u{5b9}\u{5a3}מ\u{5b9}ת\u{5b7}\u{594}י\u{5b4}ם ל\u{5b0}מ\u{5b5}\u{596}י ה\u{5b7}ב\u{5bc}\u{5b0}ר\u{5b5}כ\u{5b8}\u{5a3}ה ה\u{5b7}י\u{5b0}ש\u{5c1}\u{5b8}נ\u{5b8}\u{591}ה ו\u{5b0}ל\u{5b9}\u{5a4}א ה\u{5b4}ב\u{5bc}\u{5b7}ט\u{5b0}ת\u{5bc}\u{5b6}ם\u{599} א\u{5b6}ל־ע\u{5b9}ש\u{5c2}\u{5b6}\u{594}יה\u{5b8} ו\u{5b0}י\u{5b9}צ\u{5b0}ר\u{5b8}\u{5a5}ה\u{5bc} מ\u{5b5}ר\u{5b8}ח\u{596}ו\u{5b9}ק ל\u{5b9}\u{5a5}א ר\u{5b0}א\u{5b4}ית\u{5b6}\u{5bd}ם׃".to_string(),
        },
    ),
];
        let surface_form = SurfaceBaseText::from_blocks(&content);
        assert_eq!(
            surface_form.indexmap.last().unwrap(),
            &SurfaceIndex {
                position_in_raw: 1157,
                block_position: 21,
                position_in_block: 14,
            }
        );
    }
}
