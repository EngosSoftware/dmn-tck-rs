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

const XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";

const NODE_COMPUTED: &str = "computed";
const NODE_DESCRIPTION: &str = "description";
const NODE_EXPECTED: &str = "expected";
const NODE_INPUT_NODE: &str = "inputNode";
const NODE_LABELS: &str = "labels";
const NODE_VALUE: &str = "value";
const NODE_LABEL: &str = "label";
const NODE_MODEL_NAME: &str = "modelName";
const NODE_RESULT_NODE: &str = "resultNode";
const NODE_TEST_CASE: &str = "testCase";
const NODE_TEST_CASES: &str = "testCases";

const ATTR_NAME: &str = "name";
const ATTR_NIL: &str = "nil";
const ATTR_TYPE: &str = "type";

/// Test cases.
#[derive(Debug)]
pub struct TestCases {
  pub model_name: Option<String>,
  pub labels: Vec<String>,
  pub test_cases: Vec<TestCase>,
}

/// Single test case.
#[derive(Debug)]
pub struct TestCase {
  pub description: Option<String>,
  pub input_nodes: Vec<InputNode>,
  pub result_nodes: Vec<ResultNode>,
}

/// Input node defined for test case.
#[derive(Debug)]
pub struct InputNode {
  pub name: String,
  pub value: Option<ValueType>,
}

/// Result node expected by test case.
#[derive(Debug)]
pub struct ResultNode {
  pub name: String,
  pub expected: Option<ValueType>,
  pub computed: Option<ValueType>,
}

/// Value representing single result of a test case.
#[derive(Debug)]
pub struct Value {
  /// XSI type of the value.
  pub xsi_type: Option<String>,
  /// XSI nil value.
  pub xsi_nil: Option<String>,
  /// Optional, textual representation of the value.
  pub text: Option<String>,
}

/// Value representing complex result of a test case.
#[derive(Debug)]
pub struct Component {}

/// Value representing a list as a result of a test case.
#[derive(Debug)]
pub struct List {}

/// Types of values representing a result of a test case.
#[derive(Debug)]
pub enum ValueType {
  Value(Value),
  Component(Component),
  List(List),
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
        Err(XmlExpectedMandatoryNode(NODE_TEST_CASES.to_string()))
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
    model_name: optional_child_required_content(node, NODE_MODEL_NAME)?,
    labels: parse_labels(node)?,
    test_cases: parse_test_cases(node)?,
  })
}

/// Parses all labels.
fn parse_labels(node: &Node) -> Result<Vec<String>, RunnerError> {
  let mut items = vec![];
  if let Some(labels_node) = node.children().find(|n| n.tag_name().name() == NODE_LABELS) {
    for ref label_node in labels_node.children().filter(|n| n.tag_name().name() == NODE_LABEL) {
      items.push(required_content(label_node)?)
    }
  }
  Ok(items)
}

/// Parses all test cases.
fn parse_test_cases(node: &Node) -> Result<Vec<TestCase>, RunnerError> {
  let mut items = vec![];
  for ref test_case_node in node.children().filter(|n| n.tag_name().name() == NODE_TEST_CASE) {
    items.push(TestCase {
      description: optional_child_required_content(test_case_node, NODE_DESCRIPTION)?,
      input_nodes: parse_input_nodes(test_case_node)?,
      result_nodes: parse_result_nodes(test_case_node)?,
    })
  }
  Ok(items)
}

/// Parses input nodes defined for test case.
fn parse_input_nodes(node: &Node) -> Result<Vec<InputNode>, RunnerError> {
  let mut items = vec![];
  for ref input_node in node.children().filter(|n| n.tag_name().name() == NODE_INPUT_NODE) {
    items.push(InputNode {
      name: required_attribute(input_node, ATTR_NAME)?,
      value: parse_value_type(input_node)?,
    })
  }
  Ok(items)
}

/// Parses result nodes expected by test case.
fn parse_result_nodes(node: &Node) -> Result<Vec<ResultNode>, RunnerError> {
  let mut items = vec![];
  for ref result_node in node.children().filter(|n| n.tag_name().name() == NODE_RESULT_NODE) {
    items.push(ResultNode {
      name: required_attribute(result_node, ATTR_NAME)?,
      expected: parse_child_value_type(result_node, NODE_EXPECTED)?,
      computed: parse_child_value_type(result_node, NODE_COMPUTED)?,
    })
  }
  Ok(items)
}

/// Parses value type.
fn parse_value_type(node: &Node) -> Result<Option<ValueType>, RunnerError> {
  if let Ok(Some(value)) = parse_value(node) {
    return Ok(Some(ValueType::Value(value)));
  }
  Ok(None)
}

/// Parses value type from child node.
fn parse_child_value_type(node: &Node, child_name: &str) -> Result<Option<ValueType>, RunnerError> {
  if let Some(ref child_node) = node.children().find(|n| n.tag_name().name() == child_name) {
    if let Ok(Some(value)) = parse_value(child_node) {
      return Ok(Some(ValueType::Value(value)));
    }
  }
  Ok(None)
}

/// Parses single value definition.
fn parse_value(node: &Node) -> Result<Option<Value>, RunnerError> {
  if let Some(ref value_node) = node.children().find(|n| n.tag_name().name() == NODE_VALUE) {
    Ok(Some(Value {
      xsi_type: optional_xsi_attribute(value_node, ATTR_TYPE),
      xsi_nil: optional_xsi_attribute(value_node, ATTR_NIL),
      text: optional_content(value_node),
    }))
  } else {
    Ok(None)
  }
}

/// XML utility function that returns the value of the required attribute or an error.
fn required_attribute(node: &Node, attr_name: &str) -> Result<String, RunnerError> {
  if let Some(attr_value) = node.attribute(attr_name) {
    Ok(attr_value.to_string())
  } else {
    Err(XmlExpectedMandatoryAttribute(attr_name.to_string()))
  }
}

/// XML utility function that returns the value of the optional `xsi:` attribute.
fn optional_xsi_attribute(node: &Node, attr_name: &str) -> Option<String> {
  if let Some(attr_value) = node.attribute((XSI, attr_name)) {
    Some(attr_value.to_string())
  } else {
    None
  }
}

/// XML utility function that returns required textual content from the specified node.
fn required_content(node: &Node) -> Result<String, RunnerError> {
  if let Some(text) = node.text() {
    Ok(text.to_string())
  } else {
    Err(XmlExpectedMandatoryTextContent(node.tag_name().name().to_string()))
  }
}

/// XML utility function that returns optional textual content of the node.
fn optional_content(node: &Node) -> Option<String> {
  if let Some(text) = node.text() {
    Some(text.to_string())
  } else {
    None
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
