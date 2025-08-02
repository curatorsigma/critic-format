//! Critic-Format - used to define the XML schema used in
//! [critic](https://github.com/curatorsigma/critic).

use denorm::NormalizationError;
use destream::StreamError;
use streamed::Manuscript;
pub mod denorm;
pub mod destream;
pub mod normalized;
pub mod schema;
pub mod streamed;

#[derive(Debug)]
pub enum ConversionError {
    Stream(StreamError),
    DeStream(StreamError),
    Norm(NormalizationError),
    DeNorm(NormalizationError),
    Ser(quick_xml::SeError),
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

// direct conversion to/from XML in string form
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

// direct conversion to/from XML in string form
pub fn from_xml(buf_reader: impl std::io::BufRead) -> Result<Manuscript, ConversionError> {
    let ds: crate::schema::Tei = quick_xml::de::from_reader(buf_reader)
        .map_err(ConversionError::DeSer)?;
    let normalized: crate::normalized::Manuscript =
        ds.try_into().map_err(ConversionError::Norm)?;
    normalized
        .try_into()
        .map_err(ConversionError::Stream)
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
}
