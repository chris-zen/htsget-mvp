use htsget_search::htsget::HtsGetError as HtsGetSearchError;
use serde::Serialize;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, HtsGetError>;

#[derive(Error, Debug, PartialEq)]
pub enum HtsGetError {
  #[error("InvalidAuthentication")]
  InvalidAuthentication(String),
  #[error("PermissionDenied")]
  PermissionDenied(String),
  #[error("NotFound")]
  NotFound(String),
  #[error("PayloadTooLarge")]
  PayloadTooLarge(String),
  #[error("UnsupportedFormat")]
  UnsupportedFormat(String),
  #[error("InvalidInput")]
  InvalidInput(String),
  #[error("InvalidRange")]
  InvalidRange(String),
}

#[derive(Serialize)]
struct JsonHtsGetError {
  error: String,
  message: String,
}

impl HtsGetError {
  pub fn to_json_representation(&self) -> (String, u16) {
    let (message, status_code) = match self {
      HtsGetError::InvalidAuthentication(s) => (s, 401),
      HtsGetError::PermissionDenied(s) => (s, 403),
      HtsGetError::NotFound(s) => (s, 404),
      HtsGetError::PayloadTooLarge(s) => (s, 413),
      HtsGetError::UnsupportedFormat(s) => (s, 400),
      HtsGetError::InvalidInput(s) => (s, 400),
      HtsGetError::InvalidRange(s) => (s, 400),
    };
    (
      serde_json::to_string_pretty(&JsonHtsGetError {
        error: self.to_string(),
        message: message.clone(),
      })
      .expect("Internal error while converting error to json"),
      status_code,
    )
  }
}

impl From<HtsGetSearchError> for HtsGetError {
  fn from(error: HtsGetSearchError) -> Self {
    match error {
      HtsGetSearchError::NotFound(s) => HtsGetError::NotFound(s),
      HtsGetSearchError::UnsupportedFormat(s) => HtsGetError::UnsupportedFormat(s),
      HtsGetSearchError::InvalidInput(s) => HtsGetError::InvalidInput(s),
      HtsGetSearchError::InvalidRange(s) => HtsGetError::InvalidRange(s),
      HtsGetSearchError::IoError(_) => HtsGetError::NotFound("There was an IO error".to_string()),
      HtsGetSearchError::ParseError(_) => {
        HtsGetError::NotFound("The requested content couldn't be parsed correctly".to_string())
      }
    }
  }
}