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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use http::Uri;
use reqwest::blocking::Client;

use crate::dto::{ExpectedValueDto, InputNodeDto, ResultDto, ValueDto};
use crate::errors::RunnerError;
use crate::model::parse_from_file;
use crate::validator::validate_test_cases_file;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};

mod config;
mod dto;
mod errors;
mod model;
#[cfg(test)]
mod tests;
mod validator;

static SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static FAILURE_COUNT: AtomicU64 = AtomicU64::new(0);
static OTHER_COUNT: AtomicU64 = AtomicU64::new(0);

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
  /// Input values.
  #[serde(rename = "input")]
  input: Vec<InputNodeDto>,
}

/// Main entrypoint of the runner.
fn main() {
  let config = config::get();
  let dir_path = Path::new(&config.test_cases_dir_path);
  if dir_path.exists() && dir_path.is_dir() {
    println!("Starting DMN TCK runner...");
    println!("Searching DMN files in directory: {}", dir_path.display());
    let client = reqwest::blocking::Client::new();
    let count = process_dmn_files(dir_path, &client, &config.deploy_url);
    println!("Deployed {} *.dmn files.", count);
    let mut writer = get_writer();
    if let Ok(count) = process_xml_files(&mut writer, dir_path, &client, &config.evaluate_url) {
      println!("Processed {} *.xml files.", count);
    }
    writer.flush().expect("flushing output file failed");
    let success_count = SUCCESS_COUNT.load(Ordering::Relaxed);
    let failure_count = FAILURE_COUNT.load(Ordering::Relaxed);
    let other_count = OTHER_COUNT.load(Ordering::Relaxed);
    println!("-----------------");
    println!("    Total: {}", success_count + failure_count + other_count);
    println!("  Success: {}", success_count);
    println!("  Failure: {}", failure_count);
    println!("    Other: {}", other_count);
  } else {
    usage();
  }
}

fn process_dmn_files(path: &Path, client: &Client, deploy_url: &str) -> u64 {
  let mut count = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(dir_entry) = entry {
        let path = dir_entry.path();
        if path.is_dir() {
          count += process_dmn_files(&path, client, deploy_url);
        } else if let Some(ext) = path.extension() {
          if ext == "dmn" {
            deploy_dmn_definitions(&path, client, deploy_url);
            count += 1;
          }
        }
      }
    }
  }
  count
}

fn deploy_dmn_definitions(path: &PathBuf, client: &Client, deploy_url: &str) {
  if let Ok(canonical) = fs::canonicalize(path) {
    if let Some(p_and_q) = canonical.to_str() {
      if let Ok(file_href) = Uri::builder()
        .scheme("file")
        .authority("localhost")
        .path_and_query(p_and_q)
        .build()
      {
        println!("Deploying: {}", file_href);
        if let Ok(content) = fs::read_to_string(canonical) {
          let params = DeployParams {
            file: file_href.to_string(),
            content: base64::encode(content),
          };
          match client.post(deploy_url).json(&params).send() {
            Ok(response) => match response.text() {
              Ok(text) => {
                println!("{}", text);
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

fn process_xml_files(
  writer: &mut BufWriter<File>,
  path: &Path,
  client: &Client,
  evaluate_url: &str,
) -> Result<u64, RunnerError> {
  let mut count = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(dir_entry) = entry {
        let path = dir_entry.path();
        if path.is_dir() {
          count += process_xml_files(writer, &path, client, evaluate_url)?;
        } else if let Some(ext) = path.extension() {
          if ext == "xml" {
            execute_tests(writer, &path, client, evaluate_url)?;
            count += 1;
          }
        }
      }
    }
  }
  Ok(count)
}

fn execute_tests(
  writer: &mut BufWriter<File>,
  path: &Path,
  client: &Client,
  evaluate_url: &str,
) -> Result<(), RunnerError> {
  println!("Processing file: {}", path.display());
  print!("Validating...");
  validate_test_cases_file(&path)?;
  print!("OK");
  print!(",  Parsing...");
  let test_cases = parse_from_file(path)?;
  println!("OK");
  let empty_id = String::new();
  for test_case in &test_cases.test_cases {
    let test_id = test_case.id.as_ref().unwrap_or(&empty_id);
    for result_node in &test_case.result_nodes {
      let name = &result_node.name;
      let artifact = match &result_node.typ {
        Some(typ) => typ.clone(),
        _ => format!("{:?}", test_case.typ),
      };
      print!(
        "Executing test case: {}, result name: '{}', artifact: '{}'\n",
        test_id, name, artifact
      );
      let params = EvaluateParams {
        artifact,
        name: name.clone(),
        input: test_case.input_nodes.iter().map(InputNodeDto::from).collect(),
      };
      match client.post(evaluate_url).json(&params).send() {
        Ok(response) => match response.json::<ResultDto<ExpectedValueDto>>() {
          Ok(result) => {
            if let Some(data) = result.data {
              if let Some(actual_dto) = data.value {
                if let Some(expected) = &result_node.expected {
                  let expected_dto = ValueDto::from(expected);
                  if actual_dto == expected_dto {
                    write_line(writer, &path, &test_id, "SUCCESS", "");
                  } else {
                    let remarks = format!("actual <> expected : {:?} <<>> {:?}", actual_dto, expected_dto);
                    write_line(writer, &path, &test_id, "FAILURE", &remarks);
                  }
                } else {
                  write_line(writer, &path, &test_id, "FAILURE", "no expected value");
                }
              } else {
                write_line(writer, &path, &test_id, "FAILURE", "no actual value");
              }
            } else if let Some(errors) = result.errors {
              let remarks = errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ");
              write_line(writer, &path, &test_id, "FAILURE", &remarks);
            } else {
              write_line(writer, &path, &test_id, "FAILURE", format!("{:?}", result).as_str());
            }
          }
          Err(reason) => {
            write_line(writer, &path, &test_id, "FAILURE", &reason.to_string());
          }
        },
        Err(reason) => {
          write_line(writer, &path, &test_id, "FAILURE", &reason.to_string());
        }
      }
    }
  }
  Ok(())
}

fn write_line(writer: &mut BufWriter<File>, path: &Path, test_id: &str, test_result: &str, remarks: &str) {
  let dir_name = path
    .parent()
    .expect("taking parent path failed")
    .to_str()
    .expect("converting parent path to string failed");
  let file_name = path
    .file_stem()
    .expect("taking file stem failed")
    .to_str()
    .expect("converting file stem to string failed");
  writeln!(
    writer,
    r#""{}","{}","{}","{}","{}""#,
    dir_name, file_name, test_id, test_result, remarks
  )
  .expect("writing output line failed");
  match test_result.to_lowercase().as_str() {
    "failure" => {
      FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
      eprintln!("FAILURE: {}", remarks);
    }
    "success" => {
      SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
      println!("SUCCESS");
    }
    _ => {
      OTHER_COUNT.fetch_add(1, Ordering::Relaxed);
      println!("{}: {}", test_result, remarks);
    }
  }
}

fn get_writer() -> BufWriter<File> {
  let file = File::create("report.csv").expect("creating output file failed");
  BufWriter::new(file)
}

fn usage() {
  println!("Do the help, please...")
}
