pub use htsget_config::config::{Config, DataServerConfig, ServiceInfo, TicketServerConfig};
#[cfg(feature = "s3-storage")]
pub use htsget_config::regex_resolver::aws::S3Resolver;
pub use htsget_config::regex_resolver::{
  LocalResolver, QueryAllowed, RegexResolver, Resolver, StorageType,
};

pub mod htsget;
pub mod storage;
