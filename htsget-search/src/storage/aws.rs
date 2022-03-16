//! Module providing an implementation for the [Storage] trait using Amazon's S3 object storage service.
use std::io::{Cursor, Error, ErrorKind, Read, SeekFrom};
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use aws_config;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::input::GetObjectInput;
use aws_sdk_s3::presigning::config::PresigningConfig;
use aws_sdk_s3::{ByteStream, Client as S3Client, Config, Region};
use bytes::Bytes;
use futures::{AsyncRead, TryStreamExt, TryFutureExt, AsyncSeek};
use futures::stream::IntoAsyncRead;
use regex::Regex;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};
use crate::htsget::{HtsGetError, Url};
use crate::storage::async_storage::AsyncStorage;
use crate::storage::aws::s3_url::parse_s3_url;
use crate::storage::StorageError::InvalidKey;
use log::{trace};
use tokio::io::BufReader;
use tokio_util::io::StreamReader;
use crate::storage::StorageError;
use super::{GetOptions, Result, UrlOptions};

mod s3_testing_helper;
mod s3_url;

//use crate::storage::s3_testing::fs_write_object;

enum Retrieval {
  Immediate,
  Delayed,
}

enum AwsS3StorageTier {
  Standard(Retrieval),
  StandardIa(Retrieval),
  OnezoneIa(Retrieval),
  Glacier(Retrieval),     // ~24-48 hours
  DeepArchive(Retrieval), // ~48 hours
}

/// Implementation for the [Storage] trait utilising data from an S3 bucket.
pub struct AwsS3Storage {
  client: S3Client,
  bucket: String,
}

impl AwsS3Storage {
  pub fn new(client: S3Client, bucket: String) -> Self {
    AwsS3Storage { client, bucket }
  }

  async fn s3_presign_url(client: S3Client, bucket: String, key: String) -> Result<String> {
    let expires_in = Duration::from_secs(900);

    let region_provider = RegionProviderChain::first_try("ap-southeast-2")
      .or_default_provider()
      .or_else(Region::new("us-east-1"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;

    // Presigned requests can be made with the client directly
    let presigned_request = client
      .get_object()
      .bucket(&bucket)
      .key(&key)
      .presigned(PresigningConfig::expires_in(expires_in).unwrap())
      .await;

    // Or, they can be made directly from an operation input
    let presigned_request = GetObjectInput::builder()
      .bucket(bucket)
      .key(key)
      .build()
      .unwrap()
      .presigned(
        &Config::from(&shared_config),
        PresigningConfig::expires_in(expires_in).unwrap(),
      )
      .await;

    Ok(presigned_request.unwrap().uri().to_string())
  }

  async fn s3_head(client: S3Client, bucket: String, key: String) -> Result<u64> {
    let content_length = client
      .head_object()
      .bucket(bucket)
      .key(key)
      .send()
      .await
      .unwrap()
      .content_length as u64;

    dbg!(content_length);
    Ok(content_length)
  }

  async fn get_storage_tier(s3_url: String) -> Result<AwsS3StorageTier> {
    // 1. S3 head request to object
    // 2. Return status
    // Similar (Java) code I wrote here: https://github.com/igvteam/igv/blob/master/src/main/java/org/broad/igv/util/AmazonUtils.java#L257
    // Or with AWS cli with: $ aws s3api head-object --bucket awsexamplebucket --key dir1/example.obj
    unimplemented!();
  }

  async fn stream_from<K: AsRef<str> + Send>(&self, key: K, options: GetOptions) -> Result<Box<dyn tokio::io::AsyncRead>> {
    let s3path = PathBuf::from(&self.bucket).join(key.as_ref());

    let response = reqwest::get("http://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/gambian_genome_variation_project/release/20200217_biallelic_SNV/ALL_GGVP.chr20.shapeit2_integrated_snvindels_v1b_20200120.GRCh38.phased.vcf.gz.tbi")
      .await?
      .error_for_status()?;

    let response_stream = response.bytes_stream();

    let response_reader = response_stream
      .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
      .into_async_read();

    // Convert the futures::io::AsyncRead into a tokio::io::AsyncRead.
    let mut download = response_reader.compat();

    Ok(Box::new(download))
  }

  async fn get_content<K: AsRef<str> + Send>(&self, key: K, options: GetOptions) -> Result<Bytes> {
    let key = key.as_ref();

    // It would be nice to use a ready-made type with a ByteStream that implements AsyncRead + AsyncSeek
    // in order to avoid reading the whole byte buffer into memory. A custom type could be made similar to
    // https://users.rust-lang.org/t/what-to-pin-when-implementing-asyncread/63019/2 which could be based off
    // StreamReader.
    let response = self.client
      .get_object()
      .bucket(&self.bucket)
      .key(key)
      .send()
      .await
      .map_err(|err| StorageError::AwsError(err.to_string(), key.to_string()))?
      .body
      .collect()
      .await
      .map_err(|err| StorageError::AwsError(err.to_string(), key.to_string()))?
      .into_bytes();
    Ok(response)
  }
}

// TODO: Determine if all three trait methods require Retrievavility testing before
// reaching out to actual S3 objects or just the "head" operation.
// i.e: Should we even return a presigned URL if the object is not immediately retrievable?`
#[async_trait]
impl AsyncStorage for AwsS3Storage {
  type Streamable = BufReader<Cursor<Bytes>>;

  /// Returns the S3 url (s3://bucket/key) for the given path (key).
  async fn get<K: AsRef<str> + Send>(&self, key: K, _options: GetOptions) -> Result<BufReader<Cursor<Bytes>>> {
    let response = self.get_content(key, _options).await?;
    let cursor = Cursor::new(response);
    let reader = tokio::io::BufReader::new(cursor);
    Ok(reader)
  }

  /// Returns a S3-presigned htsget URL
  async fn url<K: AsRef<str> + Send>(&self, key: K, options: UrlOptions) -> Result<Url> {
    let region_provider = RegionProviderChain::first_try("ap-southeast-2")
      .or_default_provider()
      .or_else(Region::new("us-east-1"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;

    let client = S3Client::new(&shared_config);

    let presigned_url =
      AwsS3Storage::s3_presign_url(client, self.bucket.clone(), key.as_ref().to_string());
    let htsget_url = Url::new(presigned_url.await?);
    Ok(htsget_url)
  }

  /// Returns the size of the S3 object in bytes.
  async fn head<K: AsRef<str> + Send>(&self, key: K) -> Result<u64> {
    let region_provider = RegionProviderChain::first_try("ap-southeast-2")
      .or_default_provider()
      .or_else(Region::new("us-east-1"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;

    let key: &str = key.as_ref(); // input URI or path, not S3 key... the trait naming is a bit misleading
    let client = S3Client::new(&shared_config);

    let (bucket, s3key, _) = parse_s3_url(key)?;

    let object_bytes = AwsS3Storage::s3_head(client, self.bucket.clone(), s3key).await?;
    Ok(object_bytes)
  }
}

#[cfg(test)]
mod tests {
  use crate::storage::aws::s3_url::parse_s3_url;
  use hyper::{Body, Method, StatusCode};
  use s3_server::headers::HeaderValue;
  use s3_server::headers::X_AMZ_CONTENT_SHA256;

  use crate::storage::aws::s3_testing_helper::fs_write_object;
  use crate::storage::aws::s3_testing_helper::recv_body_string;
  use crate::storage::aws::s3_testing_helper::setup_service;

  use super::*;

  type Request = hyper::Request<hyper::Body>;

  async fn aws_s3_client() -> S3Client {
    let region_provider = RegionProviderChain::first_try("ap-southeast-2")
      .or_default_provider()
      .or_else(Region::new("us-east-1"));

    let shared_config = aws_config::from_env().region(region_provider).load().await;

    S3Client::new(&shared_config)
  }

  #[tokio::test]
  async fn test_get_htsget_url_from_s3() {
    let s3_storage = AwsS3Storage::new(aws_s3_client().await, String::from("bucket"));

    let htsget_url = s3_storage.url("key", UrlOptions::default()).await.unwrap();

    dbg!(&htsget_url);
    assert!(htsget_url.url.contains("X-Amz-Signature"));
  }

  // #[tokio::test]
  // async fn test_get_head_bytes_from_s3() {
  //   // Tilt up the local S3 server...
  //   let (root, service) = setup_service().unwrap();

  //   let bucket = "asd";
  //   let key = "qwe";
  //   let content = "Hello World!";

  //   fs_write_object(root, bucket, key, content).unwrap();

  //   let mut req = Request::new(Body::empty());
  //   *req.method_mut() = Method::GET;
  //   *req.uri_mut() = format!("http://localhost:8014/{}/{}", bucket, key)
  //       .parse()
  //       .unwrap();
  //   req.headers_mut().insert(
  //       X_AMZ_CONTENT_SHA256.clone(),
  //       HeaderValue::from_static("UNSIGNED-PAYLOAD"),
  //   );

  //   let mut res = service.hyper_call(req).await.unwrap();
  //   let body = recv_body_string(&mut res).await.unwrap();

  //   // TODO: Find an aws_sdk_rust equivalent? Not sure this exists :_S
  //   // let local_region = Region::Custom {
  //   //   endpoint: "http://localhost:8014".to_owned(),
  //   //   name: "local".to_owned(),
  //   // };

  //   let s3_storage = AwsS3Storage::new(
  //     aws_s3_client().await,
  //     bucket.to_string(),
  //     key.to_string(),
  //   );

  //   let obj_head = format!("http://localhost:8014/{}/{}", bucket, key);
  //   //dbg!(&obj_head);
  //   let htsget_head = s3_storage.head(obj_head).await.unwrap();

  //   // assert_eq!(res.status(), StatusCode::OK);
  //   // assert_eq!(body, content);
  // }

  #[tokio::test]
  async fn test_get_local_s3_server_object() {
    let (root, service) = setup_service().unwrap();

    let bucket = "asd";
    let key = "qwe";
    let content = "Hello World!";

    fs_write_object(root, bucket, key, content).unwrap();

    let mut req = Request::new(Body::empty());
    *req.method_mut() = Method::GET;
    *req.uri_mut() = format!("http://localhost:8014/{}/{}", bucket, key)
      .parse()
      .unwrap();
    req.headers_mut().insert(
      X_AMZ_CONTENT_SHA256.clone(),
      HeaderValue::from_static("UNSIGNED-PAYLOAD"),
    );

    let mut res = service.hyper_call(req).await.unwrap();
    let body = recv_body_string(&mut res).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body, content);
  }

  #[tokio::test]
  async fn local_s3_server_returns_htsget_url() {
    let (root, service) = setup_service().unwrap();

    let bucket = "bucket";
    let key = "key";
    let content = "Hello World!";
  }
}
