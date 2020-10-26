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

use std::env;
use std::path::Path;

/// Main entrypoint of the runner.
fn main() {
    println!("Starting DMN TCK Rust runner...");
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    if !check_args(&args) { return; }

    let path_string = args.get(1);

    let base_path = check_path(path_string);

    if base_path.is_none() { return; }

    println!("Valid base path: {:?}", base_path.unwrap());
}

fn check_args(args: &Vec<String>) -> bool {
    if args.len() < 2 {
        println!("Runner require command line argument with path.");
        return false;
    } else if args.len() > 2 {
        println!("Runner require only one command line argument.");
        return false;
    }
    return true;
}

fn check_path(path_string: Option<&String>) -> Option<&Path> {
    if path_string.is_none() {
        println!("Path argument is empty.");
        return None;
    }
    let path = Path::new(path_string.unwrap());

    if !path.exists() || !path.is_dir() {
        println!("Path does not exists or is not a directory.");
        return None;
    }

    return Some(path);
}