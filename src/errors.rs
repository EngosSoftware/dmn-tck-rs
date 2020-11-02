#[derive(Debug, PartialEq)]
pub enum RunnerError {
  ReadingFileFailed(String),
  ParsingXMLFailed(String),
  XMLExpectedMandatoryNode(String),
  XmlExpectedMandatoryTextContent(String),
}
