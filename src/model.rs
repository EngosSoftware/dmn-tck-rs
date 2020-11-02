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

//! Test case model to be parsed from XML definition file.

use crate::errors::RunnerError;
use crate::errors::RunnerError::*;
use roxmltree::Node;
use std::fs::read_to_string;
use std::path::Path;

const NODE_DESCRIPTION: &str = "description";
const NODE_MODEL_NAME: &str = "modelName";
const NODE_TEST_CASE: &str = "testCase";
const NODE_TEST_CASES: &str = "testCases";

#[derive(Debug)]
pub struct TestCases {
  pub model_name: Option<String>,
  pub test_cases: Vec<TestCase>,
}

#[derive(Debug)]
pub struct TestCase {
  pub description: Option<String>,
}

/// Reads the XML file containing test cases.
/// This function reads the whole file into string and passes it to further processing.
pub fn parse_from_file(file_path: &Path) -> Result<TestCases, RunnerError> {
  match read_to_string(Path::new(file_path)) {
    Ok(content) => parse_from_xml(&content),
    Err(reason) => Err(ReadingFileFailed(format!("{}", reason))),
  }
}

/// Parses XML file containing test cases.
fn parse_from_xml(xml: &str) -> Result<TestCases, RunnerError> {
  match roxmltree::Document::parse(&xml) {
    Ok(document) => {
      let test_cases_node = document.root_element();
      if test_cases_node.tag_name().name() != NODE_TEST_CASES {
        Err(XMLExpectedMandatoryNode(NODE_TEST_CASES.to_string()))
      } else {
        parse_root_node(&test_cases_node)
      }
    }
    Err(reason) => Err(ParsingXMLFailed(format!("{}", reason))),
  }
}

/// Parses `testCases` node being the root element of the document.
fn parse_root_node(node: &Node) -> Result<TestCases, RunnerError> {
  Ok(TestCases {
    model_name: optional_child_required_content(&node, NODE_MODEL_NAME)?,
    test_cases: parse_test_cases(node)?,
  })
}

/// Parses all test cases.
fn parse_test_cases(node: &Node) -> Result<Vec<TestCase>, RunnerError> {
  let mut test_cases = vec![];
  for ref test_case_node in node.children().filter(|n| n.tag_name().name() == NODE_TEST_CASE) {
    test_cases.push(TestCase {
      description: optional_child_required_content(test_case_node, NODE_DESCRIPTION)?,
    })
  }
  Ok(test_cases)
}

/// XML utility function that returns required textual content from the specified node.
fn required_content(node: &Node) -> Result<String, RunnerError> {
  if let Some(text) = node.text() {
    Ok(text.to_string())
  } else {
    Err(XmlExpectedMandatoryTextContent(node.tag_name().name().to_string()))
  }
}

/// XML utility function that returns the required textual content from the optional child node.
fn optional_child_required_content(node: &Node, child_name: &str) -> Result<Option<String>, RunnerError> {
  if let Some(child_node) = node.children().find(|n| n.tag_name().name() == child_name) {
    Ok(required_content(&child_node).ok())
  } else {
    Ok(None)
  }
}
