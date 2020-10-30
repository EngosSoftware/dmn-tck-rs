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
//!
//!

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate serde_derive;

use http::Uri;
use reqwest::blocking::Client;
use std::path::{Path, PathBuf};
use std::{env, fs};

const URL: &str = "http://0.0.0.0:12000/dpl";

/// Parameters for deploying definitions from *.dmn files.
#[derive(Serialize)]
pub struct DeployDefinitionsParams {
  /// URI from which the file content is transmitted.
  #[serde(rename = "file")]
  file: String,
  /// DMN definition file content (Base64 encoded).
  #[serde(rename = "content")]
  content: String,
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
      println!("Processed {} files.", count);
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
      if let Ok(file_href) = Uri::builder()
        .scheme("file")
        .authority("localhost")
        .path_and_query(p_and_q)
        .build()
      {
        println!("Deploying: {}", file_href);
        if let Ok(content) = fs::read_to_string(canonical) {
          let params = DeployDefinitionsParams {
            file: file_href.to_string(),
            content: base64::encode(content),
          };
          match client.post(URL).json(&params).send() {
            Ok(response) => {
              println!("{:?}\n", response.text().unwrap());
            }
            Err(reason) => println!("ERROR: {:?}", reason),
          }
        }
      }
    }
  }
}

fn usage() {
  println!("Do the help, please...")
}
