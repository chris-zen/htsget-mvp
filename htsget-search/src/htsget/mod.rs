//! Module providing a representation of the HtsGet specification.
//!
//! Based on the [HtsGet Specification](https://samtools.github.io/hts-specs/htsget.html).
//!

use core::fmt;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io;
use std::io::ErrorKind;

use async_trait::async_trait;
use noodles::core::region::Interval as NoodlesInterval;
use noodles::core::Position;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinError;

use crate::storage::StorageError;

pub mod bam_search;
pub mod bcf_search;
pub mod cram_search;
pub mod from_storage;
pub mod search;
pub mod vcf_search;

type Result<T> = core::result::Result<T, HtsGetError>;

/// Trait representing a search for either `reads` or `variants` in the HtsGet specification.
#[async_trait]
pub trait HtsGet {
  async fn search(&self, query: Query) -> Result<Response>;
  fn get_supported_formats(&self) -> Vec<Format>;
  fn are_field_parameters_effective(&self) -> bool;
  fn are_tag_parameters_effective(&self) -> bool;
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HtsGetError {
  #[error("not found: {0}")]
  NotFound(String),

  #[error("unsupported Format: {0}")]
  UnsupportedFormat(String),

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("invalid range: {0}")]
  InvalidRange(String),

  #[error("io error: {0}")]
  IoError(String),

  #[error("parsing error: {0}")]
  ParseError(String),

  #[error("internal error: {0}")]
  InternalError(String),
}

impl HtsGetError {
  pub fn not_found<S: Into<String>>(message: S) -> Self {
    Self::NotFound(message.into())
  }

  pub fn unsupported_format<S: Into<String>>(format: S) -> Self {
    Self::UnsupportedFormat(format.into())
  }

  pub fn invalid_input<S: Into<String>>(message: S) -> Self {
    Self::InvalidInput(message.into())
  }

  pub fn invalid_range<S: Into<String>>(message: S) -> Self {
    Self::InvalidRange(message.into())
  }

  pub fn io_error<S: Into<String>>(message: S) -> Self {
    Self::IoError(message.into())
  }

  pub fn parse_error<S: Into<String>>(message: S) -> Self {
    Self::ParseError(message.into())
  }

  pub fn concurrency_error<S: Into<String>>(message: S) -> Self {
    Self::InternalError(message.into())
  }
}

impl From<HtsGetError> for io::Error {
  fn from(error: HtsGetError) -> Self {
    Self::new(ErrorKind::Other, error)
  }
}

impl From<StorageError> for HtsGetError {
  fn from(err: StorageError) -> Self {
    match err {
      err @ (StorageError::InvalidKey(_) | StorageError::InvalidInput(_)) => {
        Self::InvalidInput(err.to_string())
      }
      err @ StorageError::KeyNotFound(_) => Self::NotFound(err.to_string()),
      err @ StorageError::IoError(_, _) => Self::IoError(err.to_string()),
      err @ (StorageError::TicketServerError(_)
      | StorageError::InvalidUri(_)
      | StorageError::InvalidAddress(_)
      | StorageError::InternalError(_)) => Self::InternalError(err.to_string()),
      #[cfg(feature = "s3-storage")]
      err @ StorageError::AwsS3Error(_, _) => Self::IoError(err.to_string()),
    }
  }
}

impl From<JoinError> for HtsGetError {
  fn from(err: JoinError) -> Self {
    Self::concurrency_error(err.to_string())
  }
}

impl From<io::Error> for HtsGetError {
  fn from(err: io::Error) -> Self {
    Self::io_error(err.to_string())
  }
}

/// A query contains all the parameters that can be used when requesting
/// a search for either of `reads` or `variants`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
  pub id: String,
  pub format: Format,
  pub class: Class,
  /// Reference name
  pub reference_name: Option<String>,
  /// The start and end positions are 0-based. [start, end)  
  pub interval: Interval,
  pub fields: Fields,
  pub tags: Tags,
  pub no_tags: Option<Vec<String>>,
}

impl Query {
  pub fn new(id: impl Into<String>, format: Format) -> Self {
    Self {
      id: id.into(),
      format,
      class: Class::Body,
      reference_name: None,
      interval: Interval::default(),
      fields: Fields::All,
      tags: Tags::All,
      no_tags: None,
    }
  }

  pub fn with_format(mut self, format: Format) -> Self {
    self.format = format;
    self
  }

  pub fn with_class(mut self, class: Class) -> Self {
    self.class = class;
    self
  }

  pub fn with_reference_name(mut self, reference_name: impl Into<String>) -> Self {
    self.reference_name = Some(reference_name.into());
    self
  }

  pub fn with_start(mut self, start: u32) -> Self {
    self.interval.start = Some(start);
    self
  }

  pub fn with_end(mut self, end: u32) -> Self {
    self.interval.end = Some(end);
    self
  }

  pub fn with_fields(mut self, fields: Fields) -> Self {
    self.fields = fields;
    self
  }

  pub fn with_tags(mut self, tags: Tags) -> Self {
    self.tags = tags;
    self
  }

  pub fn with_no_tags(mut self, no_tags: Vec<impl Into<String>>) -> Self {
    self.no_tags = Some(no_tags.into_iter().map(|field| field.into()).collect());
    self
  }
}

/// An interval represents the start (0-based, inclusive) and end (0-based exclusive) ranges of the
/// query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Interval {
  pub start: Option<u32>,
  pub end: Option<u32>,
}

impl Interval {
  const MIN_SEQ_POSITION: usize = 1;

  fn into_one_based<F>(self, max_seq_position: F) -> Result<impl Into<NoodlesInterval>>
  where
    F: FnOnce() -> usize,
  {
    Ok(
      Self::convert_position(
        self.start,
        || Self::MIN_SEQ_POSITION,
        |value| {
          value.checked_add(1).ok_or_else(|| {
            HtsGetError::InvalidRange(format!("could not convert {} to 1-based position.", value))
          })
        },
      )?..=Self::convert_position(self.end, max_seq_position, Ok)?,
    )
  }

  /// Convert between position types.
  fn convert_position<D, F>(value: Option<u32>, default: D, convert_fn: F) -> Result<Position>
  where
    D: FnOnce() -> usize,
    F: FnOnce(u32) -> Result<u32>,
  {
    let value = value
      .map(convert_fn)
      .transpose()?
      .map(|value| {
        usize::try_from(value).map_err(|err| {
          HtsGetError::InvalidRange(format!("could not convert `u32` to `usize`: {}", err))
        })
      })
      .transpose()?
      .unwrap_or_else(default);

    Position::try_from(value).map_err(|err| {
      HtsGetError::InvalidRange(format!(
        "could not convert `{}` into `Position`: {}",
        value, err
      ))
    })
  }
}

/// An enumeration with all the possible formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Format {
  Bam,
  Cram,
  Vcf,
  Bcf,
}

// TODO Allow the user to change this.
impl Format {
  pub(crate) fn fmt_file(&self, id: &str) -> String {
    match self {
      Format::Bam => format!("{}.bam", id),
      Format::Cram => format!("{}.cram", id),
      Format::Vcf => format!("{}.vcf.gz", id),
      Format::Bcf => format!("{}.bcf", id),
    }
  }

  pub(crate) fn fmt_index(&self, id: &str) -> String {
    match self {
      Format::Bam => format!("{}.bam.bai", id),
      Format::Cram => format!("{}.cram.crai", id),
      Format::Vcf => format!("{}.vcf.gz.tbi", id),
      Format::Bcf => format!("{}.bcf.csi", id),
    }
  }

  pub(crate) fn fmt_gzi(&self, id: &str) -> Result<String> {
    match self {
      Format::Bam => Ok(format!("{}.bam.gzi", id)),
      Format::Cram => Err(HtsGetError::InternalError(
        "CRAM does not support GZI".to_string(),
      )),
      Format::Vcf => Ok(format!("{}.vcf.gz.gzi", id)),
      Format::Bcf => Ok(format!("{}.bcf.gzi", id)),
    }
  }
}

impl From<Format> for String {
  fn from(format: Format) -> Self {
    format.to_string()
  }
}

impl fmt::Display for Format {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Format::Bam => write!(f, "BAM"),
      Format::Cram => write!(f, "CRAM"),
      Format::Vcf => write!(f, "VCF"),
      Format::Bcf => write!(f, "BCF"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
  Header,
  Body,
}

/// Possible values for the fields parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fields {
  /// Include all fields
  All,
  /// List of fields to include
  List(Vec<String>),
}

/// Possible values for the tags parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tags {
  /// Include all tags
  All,
  /// List of tags to include
  List(Vec<String>),
}

/// The headers that need to be supplied when requesting data from a url.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headers(HashMap<String, String>);

impl Headers {
  pub fn new(headers: HashMap<String, String>) -> Self {
    Self(headers)
  }

  pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
    self.0.insert(key.into(), value.into());
    self
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
    self.0.insert(key.into(), value.into());
  }

  pub fn into_inner(self) -> HashMap<String, String> {
    self.0
  }

  pub fn as_ref_inner(&self) -> &HashMap<String, String> {
    &self.0
  }
}

/// A url from which raw data can be retrieved.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Url {
  pub url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub headers: Option<Headers>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub class: Option<Class>,
}

impl Url {
  pub fn new<S: Into<String>>(url: S) -> Self {
    Self {
      url: url.into(),
      headers: None,
      class: None,
    }
  }

  pub fn with_headers(mut self, headers: Headers) -> Self {
    self.headers = Some(headers).filter(|h| !h.is_empty());
    self
  }

  pub fn set_class(mut self, class: Option<Class>) -> Self {
    self.class = class;
    self
  }

  pub fn with_class(self, class: Class) -> Self {
    self.set_class(Some(class))
  }
}

/// Wrapped json response for htsget.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonResponse {
  pub htsget: Response,
}

impl JsonResponse {
  pub fn new(htsget: Response) -> Self {
    Self { htsget }
  }
}

impl From<Response> for JsonResponse {
  fn from(htsget: Response) -> Self {
    Self::new(htsget)
  }
}

/// The response for a HtsGet query.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
  pub format: Format,
  pub urls: Vec<Url>,
}

impl Response {
  pub fn new(format: Format, urls: Vec<Url>) -> Self {
    Self { format, urls }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn htsget_error_not_found() {
    let result = HtsGetError::not_found("error");
    assert!(matches!(result, HtsGetError::NotFound(message) if message == "error"));
  }

  #[test]
  fn htsget_error_unsupported_format() {
    let result = HtsGetError::unsupported_format("error");
    assert!(matches!(result, HtsGetError::UnsupportedFormat(message) if message == "error"));
  }

  #[test]
  fn htsget_error_invalid_input() {
    let result = HtsGetError::invalid_input("error");
    assert!(matches!(result, HtsGetError::InvalidInput(message) if message == "error"));
  }

  #[test]
  fn htsget_error_invalid_range() {
    let result = HtsGetError::invalid_range("error");
    assert!(matches!(result, HtsGetError::InvalidRange(message) if message == "error"));
  }

  #[test]
  fn htsget_error_io_error() {
    let result = HtsGetError::io_error("error");
    assert!(matches!(result, HtsGetError::IoError(message) if message == "error"));
  }

  #[test]
  fn htsget_error_parse_error() {
    let result = HtsGetError::parse_error("error");
    assert!(matches!(result, HtsGetError::ParseError(message) if message == "error"));
  }

  #[test]
  fn htsget_error_concurrency_error() {
    let result = HtsGetError::concurrency_error("error");
    assert!(matches!(result, HtsGetError::InternalError(message) if message == "error"));
  }

  #[test]
  fn htsget_error_from_storage_not_found() {
    let result = HtsGetError::from(StorageError::KeyNotFound("error".to_string()));
    assert!(matches!(result, HtsGetError::NotFound(_)));
  }

  #[test]
  fn htsget_error_from_storage_invalid_key() {
    let result = HtsGetError::from(StorageError::InvalidKey("error".to_string()));
    assert!(matches!(result, HtsGetError::InvalidInput(_)));
  }

  #[test]
  fn query_new() {
    let result = Query::new("NA12878", Format::Bam);
    assert_eq!(result.id, "NA12878");
  }

  #[test]
  fn query_with_format() {
    let result = Query::new("NA12878", Format::Bam);
    assert_eq!(result.format, Format::Bam);
  }

  #[test]
  fn query_with_class() {
    let result = Query::new("NA12878", Format::Bam).with_class(Class::Header);
    assert_eq!(result.class, Class::Header);
  }

  #[test]
  fn query_with_reference_name() {
    let result = Query::new("NA12878", Format::Bam).with_reference_name("chr1");
    assert_eq!(result.reference_name, Some("chr1".to_string()));
  }

  #[test]
  fn query_with_start() {
    let result = Query::new("NA12878", Format::Bam).with_start(0);
    assert_eq!(result.interval.start, Some(0));
  }

  #[test]
  fn query_with_end() {
    let result = Query::new("NA12878", Format::Bam).with_end(0);
    assert_eq!(result.interval.end, Some(0));
  }

  #[test]
  fn query_with_fields() {
    let result = Query::new("NA12878", Format::Bam)
      .with_fields(Fields::List(vec!["QNAME".to_string(), "FLAG".to_string()]));
    assert_eq!(
      result.fields,
      Fields::List(vec!["QNAME".to_string(), "FLAG".to_string()])
    );
  }

  #[test]
  fn query_with_tags() {
    let result = Query::new("NA12878", Format::Bam).with_tags(Tags::All);
    assert_eq!(result.tags, Tags::All);
  }

  #[test]
  fn query_with_no_tags() {
    let result = Query::new("NA12878", Format::Bam).with_no_tags(vec!["RG", "OQ"]);
    assert_eq!(
      result.no_tags,
      Some(vec!["RG".to_string(), "OQ".to_string()])
    );
  }

  #[test]
  fn format_from_bam() {
    let result = String::from(Format::Bam);
    assert_eq!(result, "BAM");
  }

  #[test]
  fn format_from_cram() {
    let result = String::from(Format::Cram);
    assert_eq!(result, "CRAM");
  }

  #[test]
  fn format_from_vcf() {
    let result = String::from(Format::Vcf);
    assert_eq!(result, "VCF");
  }

  #[test]
  fn format_from_bcf() {
    let result = String::from(Format::Bcf);
    assert_eq!(result, "BCF");
  }

  #[test]
  fn headers_with_header() {
    let header = Headers::new(HashMap::new()).with_header("Range", "bytes=0-1023");
    let result = header.0.get("Range");
    assert_eq!(result, Some(&"bytes=0-1023".to_string()));
  }

  #[test]
  fn headers_is_empty() {
    assert!(Headers::new(HashMap::new()).is_empty());
  }

  #[test]
  fn headers_insert() {
    let mut header = Headers::new(HashMap::new());
    header.insert("Range", "bytes=0-1023");
    let result = header.0.get("Range");
    assert_eq!(result, Some(&"bytes=0-1023".to_string()));
  }

  #[test]
  fn url_with_headers() {
    let result = Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==")
      .with_headers(Headers::new(HashMap::new()));
    assert_eq!(result.headers, None);
  }

  #[test]
  fn url_with_class() {
    let result =
      Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==").with_class(Class::Header);
    assert_eq!(result.class, Some(Class::Header));
  }

  #[test]
  fn url_set_class() {
    let result =
      Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==").set_class(Some(Class::Header));
    assert_eq!(result.class, Some(Class::Header));
  }

  #[test]
  fn url_new() {
    let result = Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==");
    assert_eq!(result.url, "data:application/vnd.ga4gh.bam;base64,QkFNAQ==");
    assert_eq!(result.headers, None);
    assert_eq!(result.class, None);
  }

  #[test]
  fn response_new() {
    let result = Response::new(
      Format::Bam,
      vec![Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==")],
    );
    assert_eq!(result.format, Format::Bam);
    assert_eq!(
      result.urls,
      vec![Url::new("data:application/vnd.ga4gh.bam;base64,QkFNAQ==")]
    );
  }
}
