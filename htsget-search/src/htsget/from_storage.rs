//! Module providing an implementation of the [HtsGet] trait using a [Storage].
//!

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use htsget_config::regex_resolver::{Resolver, StorageType};
use tokio::io::AsyncRead;
use tracing::debug;
use tracing::instrument;

use crate::htsget::search::Search;
use crate::htsget::{Format, HtsGetError};
#[cfg(feature = "s3-storage")]
use crate::storage::aws::AwsS3Storage;
use crate::storage::local::LocalStorage;
use crate::storage::UrlFormatter;
use crate::RegexResolver;
use crate::{
  htsget::bam_search::BamSearch,
  htsget::bcf_search::BcfSearch,
  htsget::cram_search::CramSearch,
  htsget::vcf_search::VcfSearch,
  htsget::{HtsGet, Query, Response, Result},
  storage::Storage,
};

/// Implementation of the [HtsGet] trait using a [Storage].
#[derive(Debug, Clone)]
pub struct HtsGetFromStorage<S> {
  storage_ref: Arc<S>,
}

#[async_trait]
impl HtsGet for Vec<RegexResolver> {
  async fn search(&self, query: Query) -> Result<Response> {
    self.as_slice().search(query).await
  }
}

#[async_trait]
impl HtsGet for &[RegexResolver] {
  async fn search(&self, query: Query) -> Result<Response> {
    for resolver in self.iter() {
      if let Some(id) = resolver.resolve_id(&query) {
        match resolver.storage_type() {
          StorageType::Local(url) => {
            let searcher = HtsGetFromStorage::local_from(url.local_path(), url.clone())?;
            return searcher.search(query.with_id(id)).await;
          }
          #[cfg(feature = "s3-storage")]
          StorageType::S3(s3) => {
            let searcher = HtsGetFromStorage::s3_from(s3.bucket().to_string()).await;
            return searcher.search(query.with_id(id)).await;
          }
          _ => {}
        }
      }
    }

    Err(HtsGetError::not_found(
      "failed to match query with resolver",
    ))
  }
}

#[async_trait]
impl<S, R> HtsGet for HtsGetFromStorage<S>
where
  R: AsyncRead + Send + Sync + Unpin,
  S: Storage<Streamable = R> + Sync + Send + 'static,
{
  #[instrument(level = "debug", skip(self))]
  async fn search(&self, query: Query) -> Result<Response> {
    debug!(format = ?query.format(), ?query, "searching {:?}, with query {:?}", query.format(), query);
    match query.format() {
      Format::Bam => BamSearch::new(self.storage()).search(query).await,
      Format::Cram => CramSearch::new(self.storage()).search(query).await,
      Format::Vcf => VcfSearch::new(self.storage()).search(query).await,
      Format::Bcf => BcfSearch::new(self.storage()).search(query).await,
    }
  }
}

impl<S> HtsGetFromStorage<S> {
  pub fn new(storage: S) -> Self {
    Self {
      storage_ref: Arc::new(storage),
    }
  }

  pub fn storage(&self) -> Arc<S> {
    Arc::clone(&self.storage_ref)
  }
}

#[cfg(feature = "s3-storage")]
impl HtsGetFromStorage<AwsS3Storage> {
  pub async fn s3_from(bucket: String) -> Self {
    HtsGetFromStorage::new(AwsS3Storage::new_with_default_config(bucket).await)
  }
}

impl<T: UrlFormatter + Send + Sync> HtsGetFromStorage<LocalStorage<T>> {
  pub fn local_from<P: AsRef<Path>>(path: P, formatter: T) -> Result<Self> {
    Ok(HtsGetFromStorage::new(LocalStorage::new(path, formatter)?))
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use std::fs;
  use std::future::Future;
  use std::path::PathBuf;

  use htsget_config::config::cors::CorsConfig;
  use tempfile::TempDir;

  use htsget_test::util::expected_bgzf_eof_data_url;

  use crate::htsget::bam_search::tests::{
    expected_url as bam_expected_url, with_local_storage as with_bam_local_storage,
  };
  use crate::htsget::vcf_search::tests::{
    expected_url as vcf_expected_url, with_local_storage as with_vcf_local_storage,
  };
  use crate::htsget::{Headers, Url};
  use crate::storage::data_server::HttpTicketFormatter;

  use super::*;

  #[tokio::test]
  async fn search_bam() {
    with_bam_local_storage(|storage| async move {
      let htsget = HtsGetFromStorage::new(Arc::try_unwrap(storage).unwrap());
      let query = Query::new("htsnexus_test_NA12878", Format::Bam);
      let response = htsget.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(bam_expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-2596770")),
          Url::new(expected_bgzf_eof_data_url()),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_vcf() {
    with_vcf_local_storage(|storage| async move {
      let htsget = HtsGetFromStorage::new(Arc::try_unwrap(storage).unwrap());
      let filename = "spec-v4.3";
      let query = Query::new(filename, Format::Vcf);
      let response = htsget.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Vcf,
        vec![
          Url::new(vcf_expected_url(filename))
            .with_headers(Headers::default().with_header("Range", "bytes=0-822")),
          Url::new(expected_bgzf_eof_data_url()),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  async fn with_local_storage_fn<F, Fut>(test: F, path: &str, file_names: Option<&[&str]>)
  where
    F: FnOnce(Arc<LocalStorage<HttpTicketFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    let mut base_path = std::env::current_dir()
      .unwrap()
      .parent()
      .unwrap()
      .join(path);

    let tmp_dir = TempDir::new().unwrap();
    if let Some(file_names) = file_names {
      for file_name in file_names {
        fs::copy(base_path.join(file_name), tmp_dir.path().join(file_name)).unwrap();
      }
      base_path = PathBuf::from(tmp_dir.path());
    }

    test(Arc::new(
      LocalStorage::new(
        base_path,
        HttpTicketFormatter::new("127.0.0.1:8081".parse().unwrap(), CorsConfig::default()),
      )
      .unwrap(),
    ))
    .await
  }

  pub(crate) async fn with_local_storage<F, Fut>(test: F, path: &str)
  where
    F: FnOnce(Arc<LocalStorage<HttpTicketFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    with_local_storage_fn(test, path, None).await;
  }

  pub(crate) async fn with_local_storage_tmp<F, Fut>(test: F, path: &str, file_names: &[&str])
  where
    F: FnOnce(Arc<LocalStorage<HttpTicketFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    with_local_storage_fn(test, path, Some(file_names)).await;
  }
}
