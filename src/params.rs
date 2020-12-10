use crate::dto::InputNodeDto;

/// Parameters for deploying definitions from *.dmn files.
#[derive(Serialize)]
pub struct DeployParams {
  /// The URI of the deployed DMN™ model.
  #[serde(rename = "source")]
  pub source: Option<String>,
  /// The content of the DMN™ model, encoded in `Base64`.
  #[serde(rename = "content")]
  pub content: Option<String>,
  /// Unique tag associated with the deployed model.
  #[serde(rename = "tag")]
  pub tag: Option<String>,
}

/// Parameters for evaluating decision artifact.
#[derive(Serialize)]
pub struct EvaluateParams {
  /// Tag of the model where the artifact will be searched.
  #[serde(rename = "tag")]
  pub tag: Option<String>,
  /// Type of decision artifact to be evaluated.
  #[serde(rename = "artifact")]
  pub artifact: String,
  /// Name of the artifact to be evaluated.
  #[serde(rename = "name")]
  pub name: String,
  /// Input values.
  #[serde(rename = "input")]
  pub input: Vec<InputNodeDto>,
}
