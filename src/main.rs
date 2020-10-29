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
use base64;

/// Main entrypoint of the runner.
fn main() {
  println!("Starting DMN TCK Rust runner...");
  let args: Vec<String> = env::args().collect();  // let args = env::args().collect::<Vec<String>>();

  println!("{:?}", args); // to jest Debug a nie Display // #[derive(Debug)]

  if !check_args(&args) { return; }

  let path_string = args.get(1);

  if let Some(p) = args.get(1) {

    let path = Path::new(p);

    println!("Valid base path: {:?}", path);

    if path.exists() && path.is_dir() {
      search_dmn_files(path);
    }

  } else {
    println!("Path argument is empty.");
  }


}

// TODO It would be better to check equality == 2 and display usage message when not true.
fn check_args(args: &Vec<String>) -> bool {
  if args.len() < 2 {
    println!("Runner require command line argument with path.");
    return false;
  }
  if args.len() > 2 {
    println!("Runner require only one command line argument.");
    return false;
  }
  return true;
}

fn search_dmn_files(path: &Path) {
  if path.is_dir() {
    for entry in fs::read_dir(path).unwrap() {
      if entry.is_ok() {
        let path = entry.unwrap().path();
        if path.is_dir() {
          search_dmn_files(&path);
        } else if path.extension().is_some() && path.extension().unwrap().eq("dmn") {
          send_content(path);
        }
      }
    }
  }
}

fn send_content(path: PathBuf) {
  println!("{}", path.to_str().unwrap());
  let content = fs::read_to_string(path);

  if content.is_ok() {
    let base64 = base64::encode(content.unwrap());

    println!("{}", base64);
  }
}