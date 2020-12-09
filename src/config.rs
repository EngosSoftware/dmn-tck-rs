use std::fs;

/// Runner configuration parameters.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationParams {
  /// Path to directory containing test cases.
  pub test_cases_dir_path: String,
  /// URL to REST service where dmn definitions will be deployed.
  pub deploy_url: String,
  /// URL to REST service where dmn definitions will be evaluated.
  pub evaluate_url: String,
  /// Path to write csv report file.
  pub report_file_path: String,
}

pub fn get() -> ConfigurationParams {
  let file_content = fs::read_to_string("runner.yml").expect("reading configuration file failed");
  serde_yaml::from_str(&file_content).expect("parsing configuration failed")
}
