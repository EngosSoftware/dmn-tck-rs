/// Data transfer object for an error.
#[derive(Debug, Deserialize)]
pub struct ErrorDto {
  /// Error details.
  #[serde(rename = "details")]
  pub details: String,
}

/// Data transfer object for a result.
#[derive(Debug, Deserialize)]
pub struct ResultDto<T> {
  /// Result containing data.
  #[serde(rename = "data")]
  pub data: Option<T>,
  /// Result containing errors.
  #[serde(rename = "errors")]
  pub errors: Option<Vec<ErrorDto>>,
}

impl<T> ResultDto<T> {
  pub fn errors_as_string(&self) -> String {
    self
      .errors
      .as_ref()
      .and_then(|v| Some(v.iter().map(|e| e.details.clone()).collect::<Vec<String>>().join(", ")))
      .unwrap_or("".to_string())
  }
}

/// Result received after successful model deployment.
#[derive(Debug, Deserialize)]
pub struct DeployResult {
  /// Unique name of the deployed model.
  #[serde(rename = "name")]
  pub name: Option<String>,
  /// Unique identifier of the deployed model.
  #[serde(rename = "id")]
  pub id: Option<String>,
  /// Unique tag associated with the deployed model.
  #[serde(rename = "tag")]
  pub tag: Option<String>,
}
