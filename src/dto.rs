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

//!

use crate::model::{Component, InputNode, List, Simple, Value};
use serde::export::Formatter;

/// Data transfer object for an error.
#[derive(Debug, Deserialize)]
pub struct ErrorDto {
  /// Error details.
  #[serde(rename = "details")]
  pub details: String,
}

/// Data transfer object for a result.
#[derive(Debug, Deserialize)]
pub struct ResultDto<T> {
  /// Result containing data.
  #[serde(rename = "data")]
  pub data: Option<T>,
  /// Result containing errors.
  #[serde(rename = "errors")]
  pub errors: Option<Vec<ErrorDto>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputNodeDto {
  #[serde(rename = "name")]
  pub name: String,
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedValueDto {
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ValueDto {
  #[serde(rename = "simple", skip_serializing_if = "Option::is_none")]
  pub simple: Option<SimpleDto>,
  #[serde(rename = "components", skip_serializing_if = "Option::is_none")]
  pub components: Option<Vec<ComponentDto>>,
  #[serde(rename = "list", skip_serializing_if = "Option::is_none")]
  pub list: Option<ListDto>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleDto {
  #[serde(rename = "type")]
  pub typ: Option<String>,
  #[serde(rename = "text")]
  pub text: Option<String>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ComponentDto {
  #[serde(rename = "name")]
  pub name: Option<String>,
  #[serde(rename = "value")]
  pub value: Option<ValueDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&Component> for ComponentDto {
  fn from(component: &Component) -> Self {
    Self {
      name: component.name.clone(),
      value: match &component.value {
        Some(value) => Some(value.into()),
        _ => None,
      },
      nil: component.nil,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ListDto {
  #[serde(rename = "items")]
  pub items: Vec<ValueDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&List> for ListDto {
  fn from(list: &List) -> Self {
    Self {
      items: list.items.iter().map(ValueDto::from).collect(),
      nil: list.nil,
    }
  }
}

impl std::fmt::Display for ErrorDto {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.details)
  }
}

impl From<&InputNode> for InputNodeDto {
  fn from(input_node: &InputNode) -> Self {
    Self {
      name: input_node.name.clone(),
      value: match &input_node.value {
        Some(value_type) => Some(value_type.into()),
        _ => None,
      },
    }
  }
}

impl From<&Simple> for SimpleDto {
  fn from(simple: &Simple) -> Self {
    Self {
      typ: simple.typ.clone(),
      text: simple.text.clone(),
      nil: simple.nil,
    }
  }
}

impl Default for ValueDto {
  fn default() -> Self {
    Self {
      simple: None,
      components: None,
      list: None,
    }
  }
}

impl From<&Value> for ValueDto {
  fn from(value: &Value) -> Self {
    match &value {
      Value::Simple(simple) => Self {
        simple: Some(simple.into()),
        ..Default::default()
      },
      Value::Components(components) => Self {
        components: Some(components.iter().map(ComponentDto::from).collect()),
        ..Default::default()
      },
      Value::List(list) => Self {
        list: Some(ListDto::from(list)),
        ..Default::default()
      },
    }
  }
}
