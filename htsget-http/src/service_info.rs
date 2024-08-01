use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing::instrument;

use htsget_config::types::Format;
use htsget_search::HtsGet;

use crate::ConfigServiceInfo;
use crate::Endpoint;

const READS_FORMATS: [&str; 2] = ["BAM", "CRAM"];
const VARIANTS_FORMATS: [&str; 2] = ["VCF", "BCF"];

/// A struct representing the information that should be present in a service-info response.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
  pub id: String,
  pub name: String,
  pub version: String,
  pub organization: Organisation,
  #[serde(rename = "type")]
  pub service_type: Type,
  pub htsget: Htsget,
  pub contact_url: String,
  pub documentation_url: String,
  pub created_at: String,
  pub updated_at: String,
  pub environment: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Organisation {
  pub name: String,
  pub url: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Type {
  pub group: String,
  pub artifact: String,
  pub version: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Htsget {
  pub datatype: String,
  pub formats: Vec<String>,
  pub fields_parameters_effective: bool,
  pub tags_parameters_effective: bool,
}

pub fn get_service_info_with(
  endpoint: Endpoint,
  supported_formats: &[Format],
  fields_effective: bool,
  tags_effective: bool,
) -> ServiceInfo {
  let htsget_info = Htsget {
    datatype: match endpoint {
      Endpoint::Reads => "reads",
      Endpoint::Variants => "variants",
    }
    .to_string(),
    formats: supported_formats
      .iter()
      .map(|format| format.to_string())
      .filter(|format| match endpoint {
        Endpoint::Reads => READS_FORMATS.contains(&format.as_str()),
        Endpoint::Variants => VARIANTS_FORMATS.contains(&format.as_str()),
      })
      .collect(),
    fields_parameters_effective: fields_effective,
    tags_parameters_effective: tags_effective,
  };

  ServiceInfo {
    id: "".to_string(),
    name: "".to_string(),
    version: "".to_string(),
    organization: Default::default(),
    service_type: Default::default(),
    htsget: htsget_info,
    contact_url: "".to_string(),
    documentation_url: "".to_string(),
    created_at: "".to_string(),
    updated_at: "".to_string(),
    environment: "".to_string(),
  }
}

#[instrument(level = "debug", skip_all)]
pub fn get_service_info_json(
  endpoint: Endpoint,
  searcher: Arc<impl HtsGet + Send + Sync + 'static>,
  config: &ConfigServiceInfo,
) -> ServiceInfo {
  debug!(endpoint = ?endpoint,"getting service-info response for endpoint");
  fill_out_service_info_json(
    get_service_info_with(
      endpoint,
      &searcher.get_supported_formats(),
      searcher.are_field_parameters_effective(),
      searcher.are_tag_parameters_effective(),
    ),
    config,
  )
}

/// Fills the service-info json with the data from the server config
fn fill_out_service_info_json(
  mut service_info_json: ServiceInfo,
  config: &ConfigServiceInfo,
) -> ServiceInfo {
  if let Some(id) = config.id() {
    service_info_json.id = id.to_string();
  }
  if let Some(name) = config.name() {
    service_info_json.name = name.to_string();
  }
  if let Some(version) = config.version() {
    service_info_json.version = version.to_string();
  }
  if let Some(organization_name) = config.organization_name() {
    service_info_json.organization.name = organization_name.to_string();
  }
  if let Some(organization_url) = config.organization_url() {
    service_info_json.organization.url = organization_url.to_string();
  }
  if let Some(contact_url) = config.contact_url() {
    service_info_json.contact_url = contact_url.to_string();
  }
  if let Some(documentation_url) = config.documentation_url() {
    service_info_json.documentation_url = documentation_url.to_string();
  }
  if let Some(created_at) = config.created_at() {
    service_info_json.created_at = created_at.to_string();
  }
  if let Some(updated_at) = config.updated_at() {
    service_info_json.updated_at = updated_at.to_string();
  }
  if let Some(environment) = config.environment() {
    service_info_json.environment = environment.to_string();
  }

  service_info_json
}
