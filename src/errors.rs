#[derive(Debug, PartialEq)]
pub enum RunnerError {
  ReadingFileFailed(String),
  ParsingXMLFailed(String),
  XmlExpectedMandatoryNode(String),
  XmlExpectedMandatoryTextContent(String),
  XmlExpectedMandatoryAttribute(String),
  XmlExpectedNode(String),
}
