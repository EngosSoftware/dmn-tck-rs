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

use std::{env, fs};
use std::path::{Path, PathBuf};
use http::Uri;

/// Main entrypoint of the runner.
fn main() {
  println!("Starting DMN TCK runner...");
  let args: Vec<String> = env::args().collect();
  if let Some(dir_name) = args.get(1) {
    let dir_path = Path::new(dir_name);
    println!("Searching DMN files in directory: {:?}", dir_path);
    if dir_path.exists() && dir_path.is_dir() {
      let count = process_dmn_files(dir_path);
      println!("Processed {} files.", count);
      return;
    }
  }
  usage();
}

fn process_dmn_files(path: &Path) -> u64 {
  let mut count = 0;
  if let Ok(entries) = fs::read_dir(path) {
    for entry in entries {
      if let Ok(dir_entry) = entry {
        let path = dir_entry.path();
        if path.is_dir() {
          count += process_dmn_files(&path);
        } else if let Some(ext) = path.extension() {
          if ext == "dmn" {
            deploy_dmn_definitions(&path);
            count += 1;
          }
        }
      }
    }
  }
  count
}

fn deploy_dmn_definitions(path: &PathBuf) {
  if let Ok(canonical) = fs::canonicalize(path) {
    if let Some(a) = canonical.to_str() {
      if let Ok(file_href) = Uri::builder().scheme("file").authority("localhost").path_and_query(a).build() {
        println!("Deploying: {}", file_href);
        if let Ok(content) = fs::read_to_string(canonical) {
          let base64 = base64::encode(content);
          println!("{}", base64);
        }
      }
    }
  }
}

fn usage() {
  println!("Do the help, please...")
}