use regex::{Error, Regex};
use serde::Deserialize;

pub trait HtsGetIdResolver {
  fn resolve_id(&self, id: &str) -> Option<String>;
}

#[derive(Debug, Deserialize)]
pub enum DataSourceType {
  LocalStorage,
  #[cfg(feature = "aws")]
  AwsS3Storage(String)
}

#[derive(Debug, Deserialize)]
pub struct DataSource {
  data_type: DataSourceType,
  #[serde(with = "serde_regex")]
  match_id_pattern: Regex,
  replacement_path: String,
}

#[derive(Debug)]
pub struct MatchedDataSource<'a> {
  data_type: &'a DataSourceType,
  path: String
}

impl DataSource {
  pub fn new(data_type: DataSourceType, match_id_pattern: &str, points_to: &str) -> Result<Self, Error> {
    Ok(DataSource {
      data_type,
      match_id_pattern: Regex::new(match_id_pattern)?,
      replacement_path: points_to.to_string(),
    })
  }
}

impl HtsGetIdResolver for DataSource {
  fn resolve_id(&self, id: &str) -> Option<String> {
    if self.match_id_pattern.is_match(id) {
      Some(self.replacement_path.replace(id, &self.replacement_path))
    } else {
      None
    }
  }
}

/// Return the first matching data source.
pub fn resolve_first<'a>(data_sources: Vec<&'a DataSource>, id: &str) -> Option<MatchedDataSource<'a>> {
  for data_source in data_sources {
    if let Some(path) = data_source.resolve_id(id) {
      return Some(MatchedDataSource {
        data_type: &data_source.data_type,
        path
      })
    }
  }
  None
}
