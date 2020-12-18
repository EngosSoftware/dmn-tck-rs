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
use std::path::Path;
use std::process::exit;

use http::Uri;
use reqwest::blocking::Client;

use crate::dto::{ExpectedValueDto, InputNodeDto, ValueDto};
use crate::errors::{Result, RunnerError};
use crate::model::parse_from_file;
use crate::params::{DeployParams, EvaluateParams};
use crate::results::{DeployResult, ResultDto};
use crate::validator::validate_test_cases_file;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};

mod config;
mod dto;
mod errors;
mod model;
mod params;
mod results;
#[cfg(test)]
mod tests;
mod validator;

static SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static FAILURE_COUNT: AtomicU64 = AtomicU64::new(0);
static OTHER_COUNT: AtomicU64 = AtomicU64::new(0);

/// Main entrypoint of the runner.
fn main() -> Result<()> {
  let config = config::get();
  let dir_path = Path::new(&config.test_cases_dir_path);
  if dir_path.exists() && dir_path.is_dir() {
    println!("Starting DMN TCK runner...");
    let client = reqwest::blocking::Client::new();
    println!("Searching DMN files in directory: {}", dir_path.display());
    let dmn_files = sorted_files(dir_path, "dmn")?;
    for dmn_file in &dmn_files {
      deploy_dmn_definitions(dmn_file, &client, &config.deploy_url)?;
    }
    println!("\n\nDeployed {} *.dmn files.\n", dmn_files.len());
    let mut writer = get_writer();
    let xml_files = sorted_files(dir_path, "xml")?;
    for xml_file in &xml_files {
      execute_tests(&mut writer, xml_file, &client, &config.evaluate_url)?;
    }
    println!("Processed {} *.xml files.", xml_files.len());
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
  Ok(())
}

fn deploy_dmn_definitions(dmn_file: &str, client: &Client, deploy_url: &str) -> Result<()> {
  if let Ok(source) = Uri::builder()
    .scheme("file")
    .authority("localhost")
    .path_and_query(dmn_file)
    .build()
  {
    println!("\nDeploying: {}", source);
    if let Ok(content) = fs::read_to_string(dmn_file) {
      let params = DeployParams {
        source: Some(source.to_string()),
        content: Some(base64::encode(content)),
        tag: Some(file_name(dmn_file)),
      };
      match client.post(deploy_url).json(&params).send() {
        Ok(response) => match response.json::<ResultDto<DeployResult>>() {
          Ok(result) => {
            if let Some(data) = result.data {
              println!(
                "SUCCESS\n    name: {}\n      id: {}\n     tag: {}",
                data.name.unwrap_or_else(|| "(no value)".to_string()),
                data.id.unwrap_or_else(|| "(no value)".to_string()),
                data.tag.unwrap_or_else(|| "(no value)".to_string())
              )
            } else if result.errors.is_some() {
              return Err(RunnerError::DeploymentFailed(result.errors_as_string()));
            } else {
              return Err(RunnerError::DeploymentFailed(format!("{:?}", result)));
            }
          }
          Err(reason) => {
            return Err(RunnerError::DeploymentFailed(format!("{:?}", reason)));
          }
        },
        Err(reason) => {
          return Err(RunnerError::DeploymentFailed(format!("{:?}", reason)));
        }
      }
    }
  }
  Ok(())
}

fn execute_tests(writer: &mut BufWriter<File>, name: &str, client: &Client, evaluate_url: &str) -> Result<()> {
  println!("\nProcessing file: {}", name);
  print!("Validating...");
  validate_test_cases_file(name)?;
  print!("OK");
  print!(",  Parsing...");
  let test_cases = parse_from_file(name)?;
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
      println!(
        "Executing test case: {}, result name: '{}', artifact: '{}'",
        test_id, name, artifact
      );
      let params = EvaluateParams {
        tag: test_cases.model_name.clone(),
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
                    write_line(writer, name, &test_id, "SUCCESS", "");
                  } else {
                    let remarks = format!("actual <> expected : {:?} <<>> {:?}", actual_dto, expected_dto);
                    write_line(writer, name, &test_id, "FAILURE", &remarks);
                  }
                } else {
                  write_line(writer, name, &test_id, "FAILURE", "no expected value");
                }
              } else {
                write_line(writer, name, &test_id, "FAILURE", "no actual value");
              }
            } else if result.errors.is_some() {
              write_line(writer, name, &test_id, "FAILURE", &result.errors_as_string());
            } else {
              write_line(writer, name, &test_id, "FAILURE", format!("{:?}", result).as_str());
            }
          }
          Err(reason) => {
            write_line(writer, name, &test_id, "FAILURE", &reason.to_string());
          }
        },
        Err(reason) => {
          write_line(writer, name, &test_id, "FAILURE", &reason.to_string());
        }
      }
    }
  }
  Ok(())
}

fn write_line(writer: &mut BufWriter<File>, name: &str, test_id: &str, test_result: &str, remarks: &str) {
  let dir_name = dir_name(name);
  let file_name = file_name(name);
  writeln!(
    writer,
    r#""{}","{}","{}","{}","{}""#,
    dir_name, file_name, test_id, test_result, remarks
  )
  .unwrap();
  match test_result.to_lowercase().as_str() {
    "failure" => {
      FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
      eprintln!("FAILURE: {}", remarks);
      exit(82);
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

fn sorted_files(path: &Path, ext: &str) -> Result<Vec<String>> {
  let mut files = search_files(path, ext)?;
  files.sort();
  Ok(files)
}

fn search_files(path: &Path, ext: &str) -> Result<Vec<String>> {
  let mut files = vec![];
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(entry) = entry {
        let path = entry.path();
        if path.is_dir() {
          files.append(search_files(&path, ext)?.as_mut());
        } else if let Some(extension) = path.extension() {
          if extension == ext {
            files.push(path.canonicalize().unwrap().display().to_string());
          }
        }
      }
    }
  }
  Ok(files)
}

/// Retrieves the parent path without file name from given `name`.
pub fn dir_name(name: &str) -> String {
  Path::new(name).parent().unwrap().to_str().unwrap().to_string()
}

/// Retrieves the file name with extension from given `name`.
pub fn file_name(name: &str) -> String {
  Path::new(name).file_name().unwrap().to_str().unwrap().to_string()
}

/// Displays usage message.
fn usage() {
  println!("Help waits for you :-)")
}
