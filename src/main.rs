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
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::any::Any;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, io};

use csv::QuoteStyle;
use http::Uri;
use reqwest::blocking::Client;
use serde_yaml::{from_str, Error};

use crate::errors::RunnerError;
use crate::model::{parse_from_file, TestCases};
use crate::validator::validate_test_cases_file;

mod dto;
mod errors;
mod model;
#[cfg(test)]
mod tests;
#[cfg_attr(target_os = "linux", path = "validator.rs")]
#[cfg_attr(not(target_os = "linux"), path = "validator_non_linux.rs")]
mod validator;
mod validator_non_linux;

// URL of the endpoint for deploying decision models
const CONFIG_DEPLOY_URL: &str = "http://0.0.0.0:12000/dpl";

// URL of the endpoint for evaluating decision artifact
const CONFIG_EVALUATE_URL: &str = "http://0.0.0.0:12000/evl";

// Names of property in runner.yml file
// const CONFIG_DEPLOY_URL_NAME: &str = "deploy_url";
// const CONFIG_EVALUATE_URL_NAME: &str = "evaluate_url";
const CONFIG_DIR_PATH_NAME: &str = "dir_path";

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
  if let Some(dir_name) = get_config() {
    let dir_path = Path::new(&dir_name);
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

/// TODO MPY: Try to parse the content of configuration yaml to struct.
fn get_config() -> Option<String> {
  if let Ok(file_content) = fs::read_to_string("runner.yml") {
    let deserialized_map: Result<BTreeMap<String, String>, Error> = from_str(&file_content);
    if let Ok(map) = deserialized_map {
      if let Some(dir_path) = map.get(CONFIG_DIR_PATH_NAME) {
        return Some(dir_path.clone());
      }
    } else {
      println!("Cannot read runner.yml")
    }
  } else {
    println!("Cannot find runner.yml")
  }
  None
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
  display_report(&path, &test_cases);
  Ok(())
}

/// TODO MPY: Now the writer is created every time the loop iterates.
/// It would be more convenient to create the CSV writer once in `main` function
/// and pass reference to it to processing functions.
/// This change will make it easier to handle writing report to file.
/// Writing to file should be buffered (buffered is faster).  
fn display_report(path: &PathBuf, test_cases: &TestCases) {
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(b',')
    .quote_style(QuoteStyle::Always)
    .double_quote(true)
    .from_writer(io::stdout());

  wtr.write_record(&["Directory name", "File name", "Test id", "Test result", "Remarks"]);
  for test_case in &test_cases.test_cases {
    let mut file_name = "";
    if let Some(path_os) = path.file_stem() {
      if let Some(path_str) = path_os.to_str() {
        file_name = path_str;
      }
    }

    let mut dir_name = "";
    if let Some(dir_str) = path.parent() {
      if let Some(dir) = dir_str.to_str() {
        dir_name = dir;
      }
    }

    let mut test_id = "";
    if let Some(id_ref) = test_case.id.as_ref() {
      test_id = id_ref.as_str();
    }

    let mut test_result = "IGNORED";
    for result in &test_case.result_nodes {
      if result.error_result {
        test_result = "FAILURE"
      } else if let Some(expected) = &result.expected {
        if let Some(computed) = &result.computed {
          if expected.type_id() == computed.type_id() {
            test_result = "SUCCESS"
          }
        }
      }
    }
    wtr.write_record(&[dir_name, file_name, test_id, test_result, ""]);
  }
  // println!("{:?}", test_cases)
}

fn usage() {
  println!("Do the help, please...")
}
