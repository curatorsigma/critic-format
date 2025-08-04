//! Critic-Format - used to define the XML schema used in
//! [critic](https://github.com/curatorsigma/critic).

use denorm::NormalizationError;
use destream::{StreamError, transform_until_page_end};
use streamed::Manuscript;
pub mod denorm;
pub mod destream;
pub mod normalized;
pub mod schema;
pub mod streamed;

/// The problems that can occur when converting XML to the internal formats.
#[derive(Debug)]
pub enum ConversionError {
    /// Failed conversion from normalized to streamed form.
    Stream(StreamError),
    /// Failed conversion from streamed to normalized form.
    DeStream(StreamError),
    /// Failed conversion from schema to normalized form.
    Norm(NormalizationError),
    /// Failed conversion from normalized to schema form.
    DeNorm(NormalizationError),
    /// Failed serialization.
    ///
    /// This usually indicates a file that was changed since being written by
    /// [`critic_format`](crate).
    Ser(quick_xml::SeError),
    /// Failed deserialization.
    ///
    /// This indicates an ill-formed XML file.
    /// It may adhere to the formal RNG schema, but not the actual TEI subspec.
    DeSer(quick_xml::DeError),
}
impl core::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Stream(e) => {
                write!(f, "Unable to stream the given normalized Data: {e}")
            }
            Self::DeStream(e) => {
                write!(f, "Unable to destream the given streamed Data: {e}")
            }
            Self::Norm(e) => {
                write!(f, "Unable to normalize the given Data: {e}")
            }
            Self::DeNorm(e) => {
                write!(f, "Unable to denormalize the given normalized Data: {e}")
            }
            Self::Ser(e) => {
                write!(f, "Unable to serialize the given Data: {e}")
            }
            Self::DeSer(e) => {
                write!(f, "Unable to deserialize the given Data: {e}")
            }
        }
    }
}
impl core::error::Error for ConversionError {}

/// Directly Convert a Manuscript to XML.
///
/// # Errors
/// Can only be [`DeStream`](ConversionError::DeStream), [`DeNorm`](ConversionError::DeNorm) and
/// [`Ser`](ConversionError::Ser) variants.
pub fn to_xml(ms: crate::streamed::Manuscript) -> Result<String, ConversionError> {
    let destreamed: crate::normalized::Manuscript =
        ms.try_into().map_err(ConversionError::DeStream)?;
    let denormed: crate::schema::Tei = destreamed.try_into().map_err(ConversionError::DeNorm)?;
    let sr = quick_xml::se::to_string_with_root("TEI", &denormed).map_err(ConversionError::Ser)?;
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<?xml-model href="https://raw.githubusercontent.com/tanakhcc/critic-format/refs/heads/master/tei_critic.rng" schematypens="http://relaxng.org/ns/structure/1.0" type="application/xml"?>{sr}"#
    ))
}

/// Directly Convert a Manuscript to XML.
///
/// # Errors
/// Can only be [`DeStream`](ConversionError::DeStream), [`DeNorm`](ConversionError::DeNorm) and
/// [`Ser`](ConversionError::Ser) variants.
/// Note that any excess blocks after the first pagebreak are discarded.
/// In particular, the first pagebreak giving this pages name MUST NOT be present
pub fn page_to_xml(
    blocks: Vec<streamed::Block>,
    pagename: String,
) -> Result<String, ConversionError> {
    let (page, _next_name) = transform_until_page_end(&mut blocks.into_iter(), pagename)
        .map_err(ConversionError::DeStream)?;
    let denormed: crate::schema::Page = page.try_into().map_err(ConversionError::DeNorm)?;
    quick_xml::se::to_string_with_root("TEI", &denormed).map_err(ConversionError::Ser)
}

/// Directly Convert from a [`BufRead`](std::io::BufRead) over XML data to a streamed Manuscript.
///
/// This combines deserialization, normalization and streaming.
///
/// # Errors
/// Can only be [`Stream`](ConversionError::Stream), [`Norm`](ConversionError::Norm) and
/// [`DeSer`](ConversionError::DeSer) variants.
pub fn from_xml(buf_reader: impl std::io::BufRead) -> Result<Manuscript, ConversionError> {
    let ds: crate::schema::Tei =
        quick_xml::de::from_reader(buf_reader).map_err(ConversionError::DeSer)?;
    let normalized: crate::normalized::Manuscript = ds.try_into().map_err(ConversionError::Norm)?;
    normalized.try_into().map_err(ConversionError::Stream)
}

/// Directly Convert from a [`BufRead`](std::io::BufRead) over XML data to a vec of streamed
/// blocks.
///
/// This combines deserialization, normalization and streaming.
/// It also guarantees that the produced blocks would form a well-formed
/// [`Page`](normalized::Page).
///
/// Note that a leading pagrebreak giving this pages name is stripped and the pagename returned
/// separately.
///
/// # Errors
/// Can only be [`Stream`](ConversionError::Stream), [`Norm`](ConversionError::Norm) and
/// [`DeSer`](ConversionError::DeSer) variants.
pub fn page_from_xml(
    buf_reader: impl std::io::BufRead,
    default_language: &str,
) -> Result<(Vec<streamed::Block>, String), ConversionError> {
    let ds: crate::schema::Page =
        quick_xml::de::from_reader(buf_reader).map_err(ConversionError::DeSer)?;
    let normalized: crate::normalized::Page = ds.try_into().map_err(ConversionError::Norm)?;
    let mut streamed = normalized.into_streamed(default_language);
    let Some(Ok(streamed::Block::Break(streamed::BreakType::Page(pagename)))) = streamed.next()
    else {
        unreachable!("into_streamed always returns at least the initial page break without error");
    };
    Ok((
        streamed
            .collect::<Result<Vec<_>, _>>()
            .map_err(ConversionError::Stream)?,
        pagename,
    ))
}

mod test {
    #[test]
    fn from_to_xml() {
        let xml = std::fs::File::open("examples/02_lines_consistent.xml").unwrap();
        let ms = super::from_xml(std::io::BufReader::new(xml)).unwrap();
        let xml_again = super::to_xml(ms.clone()).unwrap();
        let ms_again = super::from_xml(xml_again.as_bytes()).unwrap();
        assert_eq!(ms, ms_again);
    }

    #[test]
    fn page_from_to_xml() {
        let xml = std::fs::File::open("examples/08_single_page.xml").unwrap();
        let ms = super::page_from_xml(std::io::BufReader::new(xml), "grc").unwrap();
        let ms_cloned = ms.clone();
        let xml_again = super::page_to_xml(ms_cloned.0, ms_cloned.1).unwrap();
        let ms_again = super::page_from_xml(xml_again.as_bytes(), "grc").unwrap();
        assert_eq!(ms, ms_again);
    }
}
