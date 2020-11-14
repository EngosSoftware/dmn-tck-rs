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

//! Test cases for XML model.

use crate::model::{parse_from_string, TestCaseType, ValueType};
use crate::tests::{TC_0001, TC_0002, TC_0003};

#[test]
fn test_001() {
  let test_cases = parse_from_string(TC_0001).unwrap();
  assert_eq!("0001-input-data-string.dmn", test_cases.model_name.unwrap().as_str());
  assert_eq!(4, test_cases.labels.len());
  assert_eq!("Compliance Level 2", test_cases.labels[0].as_str());
  assert_eq!("Literal Expression", test_cases.labels[1].as_str());
  assert_eq!("FEEL Special-character Names", test_cases.labels[2].as_str());
  assert_eq!("Data Type: String", test_cases.labels[3].as_str());
  assert_eq!(1, test_cases.test_cases.len());
  let test_case_1 = &test_cases.test_cases[0];
  assert_eq!("001", test_case_1.id.as_ref().unwrap().as_str());
  assert_eq!(None, test_case_1.name);
  assert_eq!(TestCaseType::Decision, test_case_1.typ);
  assert_eq!(None, test_case_1.invocable_name);
  assert_eq!("Testing valid input", test_case_1.description.as_ref().unwrap().as_str());
}

#[test]
fn test_002() {
  let test_cases = parse_from_string(TC_0002).unwrap();
  assert_eq!("0008-LX-arithmetic.dmn", test_cases.model_name.unwrap().as_str());
  assert_eq!(6, test_cases.labels.len());
  assert_eq!("Compliance Level 2", test_cases.labels[0].as_str());
  assert_eq!("Literal Expression", test_cases.labels[1].as_str());
  assert_eq!("Item Definition", test_cases.labels[2].as_str());
  assert_eq!("FEEL Arithmetic", test_cases.labels[3].as_str());
  assert_eq!("Data Type: Number", test_cases.labels[4].as_str());
  assert_eq!("Data Type: Structure", test_cases.labels[5].as_str());
  assert_eq!(3, test_cases.test_cases.len());
  let test_case_1 = &test_cases.test_cases[0];
  assert_eq!("001", test_case_1.id.as_ref().unwrap().as_str());
  assert_eq!(None, test_case_1.name);
  assert_eq!(TestCaseType::Decision, test_case_1.typ);
  assert_eq!(None, test_case_1.invocable_name);
  assert_eq!(None, test_case_1.description);
  assert_eq!(1, test_case_1.input_nodes.len());
  let input_node_1 = &test_case_1.input_nodes[0];
  match &input_node_1.value {
    Some(ValueType::Components(components)) => {
      assert_eq!(3, components.len());
      let component_1 = &components[0];
      assert_eq!("principal", component_1.name.as_ref().unwrap().as_str());
      match &component_1.value {
        Some(ValueType::Simple(v)) => {
          assert_eq!("600000", v.text.as_ref().unwrap().as_str());
          assert_eq!("xsd:decimal", v.typ.as_ref().unwrap().as_str());
          assert_eq!(false, v.nil);
        }
        _ => assert!(false),
      }
      let component_2 = &components[1];
      assert_eq!("rate", component_2.name.as_ref().unwrap().as_str());
      match &component_2.value {
        Some(ValueType::Simple(v)) => {
          assert_eq!("0.0375", v.text.as_ref().unwrap().as_str());
          assert_eq!("xsd:decimal", v.typ.as_ref().unwrap().as_str());
          assert_eq!(false, v.nil);
        }
        _ => assert!(false),
      }
      let component_3 = &components[2];
      assert_eq!("termMonths", component_3.name.as_ref().unwrap().as_str());
      match &component_3.value {
        Some(ValueType::Simple(v)) => {
          assert_eq!("360", v.text.as_ref().unwrap().as_str());
          assert_eq!("xsd:decimal", v.typ.as_ref().unwrap().as_str());
          assert_eq!(false, v.nil);
        }
        _ => assert!(false),
      }
    }
    _ => assert!(false),
  };
}

#[test]
fn test_003() {
  let test_cases = parse_from_string(TC_0003).unwrap();
  assert_eq!("0012-list-functions.dmn", test_cases.model_name.unwrap().as_str());
  assert_eq!(19, test_cases.test_cases.len());
  let test_case_1 = &test_cases.test_cases[0];
  assert_eq!("001", test_case_1.id.as_ref().unwrap().as_str());
  let input_node_1 = &test_case_1.input_nodes[0];
  match &input_node_1.value {
    Some(ValueType::List(l)) => {
      assert_eq!(3, l.items.len());
      for (i, &text) in ["a", "b", "c"].iter().enumerate() {
        match &l.items[i] {
          ValueType::Simple(v) => {
            assert_eq!(text, v.text.as_ref().unwrap().as_str());
            assert_eq!("xsd:string", v.typ.as_ref().unwrap().as_str());
            assert_eq!(false, v.nil);
          }
          _ => assert!(false),
        }
      }
    }
    _ => assert!(false),
  }
}
