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

//! Runner for Decision Model and Notation™ Technology Compatibility Kit written in Rust.

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use crate::errors::RunnerError;
use crate::model::{parse_from_file, TestCases};
use crate::validator::validate_test_cases_file;
use http::Uri;
use reqwest::blocking::Client;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

mod errors;
mod model;
mod validator;

// URL of the endpoint for deploying decision models
const CONFIG_DEPLOY_URL: &str = "http://0.0.0.0:12000/dpl";

// URL of the endpoint for evaluating decision artifact
const CONFIG_EVALUATE_URL: &str = "http://0.0.0.0:12000/evl";

/// Parameters for deploying definitions from *.dmn files.
#[derive(Serialize)]
pub struct DeployParams {
  /// URI from which the file content is transmitted.
  #[serde(rename = "file")]
  file: String,
  /// DMN definition file content (Base64 encoded).
  #[serde(rename = "content")]
  content: String,
}

/// Parameters for evaluating decision artifact.
#[derive(Serialize)]
pub struct EvaluateParams {
  /// Type of decision artifact to be evaluated.
  #[serde(rename = "artifact")]
  artifact: String,
  /// Name of the artifact to be evaluated.
  #[serde(rename = "name")]
  name: String,
}

/// Main entrypoint of the runner.
fn main() {
  println!("Starting DMN TCK runner...");
  let args: Vec<String> = env::args().collect();
  if let Some(dir_name) = args.get(1) {
    let dir_path = Path::new(dir_name);
    println!("Searching DMN files in directory: {:?}", dir_path);
    if dir_path.exists() && dir_path.is_dir() {
      let client = reqwest::blocking::Client::new();
      let count = process_dmn_files(dir_path, &client);
      println!("\nProcessed {} *.dmn files.\n", count);
      if let Ok(count) = process_xml_files(dir_path, &client) {
        println!("\nProcessed {} *.xml files.\n", count);
      }
      return;
    }
  }
  usage();
}

fn process_dmn_files(path: &Path, client: &Client) -> u64 {
  let mut count = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(dir_entry) = entry {
        let path = dir_entry.path();
        if path.is_dir() {
          count += process_dmn_files(&path, client);
        } else if let Some(ext) = path.extension() {
          if ext == "dmn" {
            deploy_dmn_definitions(&path, client);
            count += 1;
          }
        }
      }
    }
  }
  count
}

fn deploy_dmn_definitions(path: &PathBuf, client: &Client) {
  if let Ok(canonical) = fs::canonicalize(path) {
    if let Some(p_and_q) = canonical.to_str() {
      if let Ok(file_href) = Uri::builder().scheme("file").authority("localhost").path_and_query(p_and_q).build() {
        println!("Deploying: {}", file_href);
        if let Ok(content) = fs::read_to_string(canonical) {
          let params = DeployParams {
            file: file_href.to_string(),
            content: base64::encode(content),
          };
          match client.post(CONFIG_DEPLOY_URL).json(&params).send() {
            Ok(response) => match response.text() {
              Ok(text) => {
                println!("{}\n", text);
                if text.contains("errors") {
                  exit(0);
                }
              }
              Err(reason) => println!("ERROR: {:?}", reason),
            },
            Err(reason) => println!("ERROR: {:?}", reason),
          }
        }
      }
    }
  }
}

fn process_xml_files(path: &Path, client: &Client) -> Result<u64, RunnerError> {
  let mut count = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(dir_entry) = entry {
        let path = dir_entry.path();
        if path.is_dir() {
          count += process_xml_files(&path, client)?;
        } else if let Some(ext) = path.extension() {
          if ext == "xml" {
            execute_tests(&path, client)?;
            count += 1;
          }
        }
      }
    }
  }
  Ok(count)
}

fn execute_tests(path: &PathBuf, client: &Client) -> Result<(), RunnerError> {
  println!("\nProcessing file: {}", path.display());
  print!("Validating...");
  validate_test_cases_file(&path)?;
  print!("OK");
  print!(",  Parsing...");
  let test_cases = parse_from_file(path)?;
  println!("OK");
  for test_case in &test_cases.test_cases {
    let id = test_case.id.as_ref().map_or("".to_string(), |a| a.clone());
    for result_node in &test_case.result_nodes {
      let name = result_node.name.clone();
      let artifact = match &result_node.typ {
        Some(t) => t.clone(),
        _ => format!("{}", test_case.typ),
      };
      print!(
        "\nEVALUATING: test case id: {:>6}, result node name: '{}', artifact: '{}'\n",
        id, name, artifact
      );
      let params = EvaluateParams { artifact, name };
      match client.post(CONFIG_EVALUATE_URL).json(&params).send() {
        Ok(response) => {
          println!("RESPONSE: {:?}\n", response.text().unwrap());
        }
        Err(reason) => println!("ERROR: {:?}", reason),
      }
      println!("OK");
    }
  }
  display_report(&test_cases);
  Ok(())
}

fn display_report(_test_cases: &TestCases) {
  // println!("{:?}", test_cases)
}

fn usage() {
  println!("Do the help, please...")
}
