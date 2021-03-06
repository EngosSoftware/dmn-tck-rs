/*
 *  Copyright 2020 Dariusz Depta Engos Software
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

//! Result and error definitions.

pub type Result<T, E = RunnerError> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum RunnerError {
  IOError(String),
  ReadingFileFailed(String),
  ParsingXMLFailed(String),
  ValidatingXMLFailed(i32),
  XmlExpectedMandatoryNode(String),
  XmlExpectedMandatoryTextContent(String),
  XmlExpectedMandatoryAttribute(String),
  DeploymentFailed(String),
}

// TODO Implement Display trait to make error reporting more verbose and user friendly.

impl From<std::io::Error> for RunnerError {
  fn from(err: std::io::Error) -> Self {
    Self::IOError(err.to_string())
  }
}
