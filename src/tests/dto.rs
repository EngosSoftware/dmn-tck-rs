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

//! Test cases for DTOs.

use crate::dto::InputNodeDto;
use crate::model::parse_from_string;
use crate::tests::{INPUTS_0001, TC_0001};

#[test]
fn test_input_nodes_value() {
  let test_cases = parse_from_string(TC_0001).unwrap();
  let test_case = &test_cases.test_cases[0];
  let input_node_dto = InputNodeDto::from(&test_case.input_nodes[0]);
  assert_eq!(INPUTS_0001, serde_json::to_string_pretty(&input_node_dto).unwrap().as_str());
}
