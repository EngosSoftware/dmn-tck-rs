use crate::model::{Component, InputNode, List, Value, ValueType};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputNodeDto {
  #[serde(rename = "name")]
  pub name: String,
  #[serde(rename = "value")]
  pub value: Option<ValueTypeDto>,
}

impl From<&InputNode> for InputNodeDto {
  fn from(input_node: &InputNode) -> Self {
    Self {
      name: input_node.name.clone(),
      value: match &input_node.value {
        Some(value_type) => Some(ValueTypeDto::from(value_type)),
        _ => None,
      },
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueTypeDto {
  #[serde(rename = "simple", skip_serializing_if = "Option::is_none")]
  pub simple: Option<ValueDto>,
  #[serde(rename = "components", skip_serializing_if = "Option::is_none")]
  pub components: Option<Vec<ComponentDto>>,
  #[serde(rename = "list", skip_serializing_if = "Option::is_none")]
  pub list: Option<ListDto>,
}

impl Default for ValueTypeDto {
  fn default() -> Self {
    Self {
      simple: None,
      components: None,
      list: None,
    }
  }
}

impl From<&ValueType> for ValueTypeDto {
  fn from(value_type: &ValueType) -> Self {
    match &value_type {
      ValueType::Simple(value) => Self {
        simple: Some(ValueDto::from(value)),
        ..Default::default()
      },
      ValueType::Components(components) => Self {
        components: Some(components.iter().map(|c| ComponentDto::from(c)).collect()),
        ..Default::default()
      },
      ValueType::List(list) => Self {
        list: Some(ListDto::from(list)),
        ..Default::default()
      },
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueDto {
  #[serde(rename = "type")]
  pub typ: Option<String>,
  #[serde(rename = "text")]
  pub text: Option<String>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&Value> for ValueDto {
  fn from(value: &Value) -> Self {
    Self {
      typ: value.typ.clone(),
      text: value.text.clone(),
      nil: value.nil,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentDto {
  #[serde(rename = "type")]
  pub name: Option<String>,
  #[serde(rename = "value")]
  pub value: Option<ValueTypeDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&Component> for ComponentDto {
  fn from(component: &Component) -> Self {
    Self {
      name: component.name.clone(),
      value: match &component.value {
        Some(value_type) => Some(ValueTypeDto::from(value_type)),
        _ => None,
      },
      nil: component.nil,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDto {
  #[serde(rename = "items")]
  pub items: Vec<ValueTypeDto>,
  #[serde(rename = "isNil")]
  pub nil: bool,
}

impl From<&List> for ListDto {
  fn from(list: &List) -> Self {
    Self {
      items: list.items.iter().map(|i| ValueTypeDto::from(i)).collect(),
      nil: list.nil,
    }
  }
}
