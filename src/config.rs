/// Runner configuration parameters.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationParams {
  /// Path to directory containing test cases.
  pub test_cases_dir_path: String,
  /// Pattern for matching test file names.
  /// Only files whose name matches the pattern will be processed.
  pub file_name_pattern: String,
  /// URL to REST service where dmn definitions will be deployed.
  pub deploy_url: String,
  /// URL to REST service where dmn definitions will be evaluated.
  pub evaluate_url: String,
  /// Path to write csv report file.
  pub report_file_path: String,
  /// Flag indicating if testing should immediately stop when a test fails.
  pub stop_on_failure: bool,
}

pub fn get() -> ConfigurationParams {
  let args: Vec<String> = std::env::args().collect();
  let cfg_file_name = if args.len() == 2 {
    args[1].as_str()
  } else {
    "runner.yml"
  };
  let err_read = format!("reading configuration file '{}' failed", cfg_file_name);
  let file_content = std::fs::read_to_string(cfg_file_name).expect(&err_read);
  let err_parse = format!("parsing configuration file '{}' failed", cfg_file_name);
  serde_yaml::from_str(&file_content).expect(&err_parse)
}
