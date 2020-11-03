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

//! Test case file validator.
//!
//! This validator is based on the code developed by Franklin Chen in project
//! https://github.com/FranklinChen/validate-xml-rust. Thanks a lot Franklin!
//!
//! To be able to link with `libxml2` library, the development version
//! must be installed, which on Fedora may be done this way:
//! ```
//! sudo yum install libxml2-devel
//! ```

use crate::errors::RunnerError;
use libc::{c_char, c_int, c_uint};
use std::ffi::CString;
use std::path::Path;

/// Fake `xmlSchema` struct from C `libxml2`.
enum XmlSchema {}

/// Free resources when schema is dropped.
impl Drop for XmlSchema {
  fn drop(&mut self) {
    unsafe {
      xmlSchemaFree(self);
    }
  }
}

/// Fake `xmlSchemaParserCtxt` struct from C `libxml2`.
enum XmlSchemaParserCtxt {}

/// Fake `xmlSchemaValidCtxt` struct from C `libxml2`.
enum XmlSchemaValidCtxt {}

/// Pointer to globally stored schema.
struct XmlSchemaPtr(pub *mut XmlSchema);

/// Required for `XmlSchemaPtr` to be thread safe.
unsafe impl Sync for XmlSchemaPtr {}

lazy_static! {
  /// Initialize global variable with XSD schema for validating test cases.
  static ref TEST_CASE_SCHEMA_PTR: XmlSchemaPtr = get_test_cases_schema();
}

#[link(name = "xml2")]
extern "C" {
  fn xmlSchemaNewMemParserCtxt(buffer: *const c_char, size: c_int) -> *mut XmlSchemaParserCtxt;
  fn xmlSchemaFreeParserCtxt(ctxt: *mut XmlSchemaParserCtxt);
  fn xmlSchemaParse(ctxt: *const XmlSchemaParserCtxt) -> *mut XmlSchema;
  fn xmlSchemaFree(schema: *mut XmlSchema);
  fn xmlSchemaNewValidCtxt(schema: *const XmlSchema) -> *mut XmlSchemaValidCtxt;
  fn xmlSchemaFreeValidCtxt(ctxt: *mut XmlSchemaValidCtxt);
  fn xmlSchemaValidateFile(ctxt: *const XmlSchemaValidCtxt, file_name: *const c_char, options: c_uint) -> c_int;
}

/// Loads schema from static resource.
fn get_test_cases_schema() -> XmlSchemaPtr {
  let schema_content = include_str!("test-cases.xsd");
  let c_to_print = CString::new(schema_content).unwrap();
  let size: c_int = schema_content.len() as i32;
  unsafe {
    let schema_parser_ctxt = xmlSchemaNewMemParserCtxt(c_to_print.as_ptr(), size);
    let schema = xmlSchemaParse(schema_parser_ctxt);
    xmlSchemaFreeParserCtxt(schema_parser_ctxt);
    XmlSchemaPtr(schema)
  }
}

/// Validates the test cases file.
pub fn validate_test_cases_file(path: &Path) -> Result<(), RunnerError> {
  let path_str = format!("{}", path.display());
  let c_path = CString::new(path_str).unwrap();
  unsafe {
    let schema_valid_ctxt = xmlSchemaNewValidCtxt(TEST_CASE_SCHEMA_PTR.0);
    let result = xmlSchemaValidateFile(schema_valid_ctxt, c_path.as_ptr(), 0);
    xmlSchemaFreeValidCtxt(schema_valid_ctxt);
    if result == 0 {
      Ok(())
    } else {
      Err(RunnerError::ValidatingXMLFailed(result))
    }
  }
}
