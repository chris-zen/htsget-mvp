use std::fmt::Debug;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::MapErr;
use futures_util::TryStreamExt;
use http::header::CONTENT_LENGTH;
use http::{HeaderMap, Method, Request, Response, Uri};
use hyper::client::HttpConnector;
use hyper::{Body, Client, Error};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use tokio_util::io::StreamReader;
use tracing::{debug, instrument};

use htsget_config::error;
use htsget_config::types::KeyType;

use crate::storage::StorageError::{InternalError, KeyNotFound, ResponseError, UrlParseError};
use crate::storage::{GetOptions, HeadOptions, RangeUrlOptions, Result, Storage, StorageError};
use crate::Url as HtsGetUrl;

/// A storage struct which derives data from HTTP URLs.
#[derive(Debug, Clone)]
pub struct UrlStorage {
  client: Client<HttpsConnector<HttpConnector>>,
  endpoint_head: Uri,
  endpoint_file: Uri,
  endpoint_index: Uri,
  response_url: Uri,
  forward_headers: bool,
  #[cfg(feature = "crypt4gh")]
  endpoint_crypt4gh_header: Option<Uri>,
}

impl UrlStorage {
  /// Construct a new UrlStorage.
  pub fn new(
    client: Client<HttpsConnector<HttpConnector>>,
    endpoint_head: Uri,
    endpoint_header: Uri,
    endpoint_index: Uri,
    response_url: Uri,
    forward_headers: bool,
    #[cfg(feature = "crypt4gh")] endpoint_crypt4gh_header: Option<Uri>,
  ) -> Self {
    Self {
      client,
      endpoint_head,
      endpoint_file: endpoint_header,
      endpoint_index,
      response_url,
      forward_headers,
      #[cfg(feature = "crypt4gh")]
      endpoint_crypt4gh_header,
    }
  }

  /// Construct a new UrlStorage with a default client.
  pub fn new_with_default_client(
    endpoint_head: Uri,
    endpoint_header: Uri,
    endpoint_index: Uri,
    response_url: Uri,
    forward_headers: bool,
    #[cfg(feature = "crypt4gh")] endpoint_crypt4gh_header: Option<Uri>,
  ) -> Self {
    Self {
      client: Client::builder().build(
        HttpsConnectorBuilder::new()
          .with_native_roots()
          .https_or_http()
          .enable_http1()
          .enable_http2()
          .build(),
      ),
      endpoint_head,
      endpoint_file: endpoint_header,
      endpoint_index,
      response_url,
      forward_headers,
      #[cfg(feature = "crypt4gh")]
      endpoint_crypt4gh_header,
    }
  }

  /// Get a url from the key.
  pub fn get_url_from_key<K: AsRef<str> + Send>(&self, key: K, endpoint: &Uri) -> Result<Uri> {
    format!("{}{}", endpoint, key.as_ref())
      .parse::<Uri>()
      .map_err(|err| UrlParseError(err.to_string()))
  }

  /// Construct and send a request
  pub async fn send_request<K: AsRef<str> + Send>(
    &self,
    key: K,
    headers: &HeaderMap,
    method: Method,
    url: &Uri,
  ) -> Result<Response<Body>> {
    let key = key.as_ref();
    let url = self.get_url_from_key(key, url)?;

    let request = Request::builder().method(method).uri(&url);

    let request = headers
      .iter()
      .fold(request, |acc, (key, value)| acc.header(key, value))
      .body(Body::empty())
      .map_err(|err| UrlParseError(err.to_string()))?;

    let response = self
      .client
      .request(request)
      .await
      .map_err(|err| KeyNotFound(format!("{} with key {}", err, key)))?;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
      Err(KeyNotFound(format!(
        "url returned {} for key {}",
        status, key
      )))
    } else {
      Ok(response)
    }
  }

  /// Construct and send a request
  pub fn format_url<K: AsRef<str> + Send>(
    &self,
    key: K,
    options: RangeUrlOptions<'_>,
    endpoint: &Uri,
  ) -> Result<HtsGetUrl> {
    let url = self.get_url_from_key(key, endpoint)?.into_parts();
    let url = Uri::from_parts(url)
      .map_err(|err| InternalError(format!("failed to convert to uri from parts: {}", err)))?;

    let mut url = HtsGetUrl::new(url.to_string());
    if self.forward_headers {
      url = url.with_headers(
        options
          .response_headers()
          .try_into()
          .map_err(|err: error::Error| StorageError::InvalidInput(err.to_string()))?,
      )
    }

    Ok(options.apply(url))
  }

  /// Get the head from the key.
  pub async fn head_key<K: AsRef<str> + Send>(
    &self,
    key: K,
    headers: &HeaderMap,
  ) -> Result<Response<Body>> {
    self
      .send_request(key, headers, Method::HEAD, &self.endpoint_head)
      .await
  }

  /// Get the key.
  pub async fn get_header<K: AsRef<str> + Send>(
    &self,
    key: K,
    headers: &HeaderMap,
  ) -> Result<Response<Body>> {
    self
      .send_request(key, headers, Method::GET, &self.endpoint_file)
      .await
  }

  /// Get the key.
  pub async fn get_index<K: AsRef<str> + Send>(
    &self,
    key: K,
    headers: &HeaderMap,
  ) -> Result<Response<Body>> {
    self
      .send_request(key, headers, Method::GET, &self.endpoint_index)
      .await
  }
}

#[async_trait]
impl Storage for UrlStorage {
  type Streamable = StreamReader<MapErr<Body, fn(Error) -> StorageError>, Bytes>;

  #[instrument(level = "trace", skip(self))]
  async fn get<K: AsRef<str> + Send + Debug>(
    &self,
    key: K,
    options: GetOptions<'_>,
  ) -> Result<Self::Streamable> {
    let key = key.as_ref().to_string();
    debug!(calling_from = ?self, key, "getting file with key {:?}", key);

    let response = match KeyType::from_ending(&key) {
      KeyType::File => {
        self
          .get_header(key.to_string(), options.request_headers())
          .await?
      }
      KeyType::Index => {
        self
          .get_index(key.to_string(), options.request_headers())
          .await?
      }
    };

    Ok(StreamReader::new(response.into_body().map_err(|err| {
      ResponseError(format!("reading body from response: {}", err))
    })))
  }

  #[instrument(level = "trace", skip(self))]
  async fn range_url<K: AsRef<str> + Send + Debug>(
    &self,
    key: K,
    options: RangeUrlOptions<'_>,
  ) -> Result<HtsGetUrl> {
    let key = key.as_ref();
    debug!(calling_from = ?self, key, "getting url with key {:?}", key);

    self.format_url(key, options, &self.response_url)
  }

  #[instrument(level = "trace", skip(self))]
  async fn head<K: AsRef<str> + Send + Debug>(
    &self,
    key: K,
    options: HeadOptions<'_>,
  ) -> Result<u64> {
    let key = key.as_ref();
    let head = self.head_key(key, options.request_headers()).await?;

    let len = head
      .headers()
      .get(CONTENT_LENGTH)
      .and_then(|content_length| content_length.to_str().ok())
      .and_then(|content_length| content_length.parse().ok())
      .ok_or_else(|| {
        ResponseError(format!(
          "failed to get content length from head response for key: {}",
          key
        ))
      })?;

    debug!(calling_from = ?self, key, len, "size of key {:?} is {}", key, len);
    Ok(len)
  }
}

#[cfg(test)]
mod tests {
  use std::future::Future;
  use std::net::TcpListener;
  use std::path::Path;
  use std::result;
  use std::str::FromStr;

  use axum::middleware::Next;
  use axum::response::Response;
  use axum::{middleware, Router};
  use http::header::AUTHORIZATION;
  use http::{HeaderName, HeaderValue, Request, StatusCode};
  use hyper::body::to_bytes;
  use tokio::io::AsyncReadExt;
  use tower_http::services::ServeDir;

  use htsget_config::types::Headers;

  use crate::storage::local::tests::create_local_test_files;

  use super::*;

  #[test]
  fn get_url_from_key() {
    let storage = UrlStorage::new(
      test_client(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://localhost:8080").unwrap(),
      true,
      #[cfg(feature = "crypt4gh")]
      None,
    );

    assert_eq!(
      storage
        .get_url_from_key(
          "assets/key1",
          &Uri::from_str("https://example.com").unwrap()
        )
        .unwrap(),
      Uri::from_str("https://example.com/assets/key1").unwrap()
    );
  }

  #[test]
  fn get_response_url_from_key() {
    let storage = UrlStorage::new(
      test_client(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://localhost:8080").unwrap(),
      true,
      #[cfg(feature = "crypt4gh")]
      None,
    );

    assert_eq!(
      storage
        .get_url_from_key(
          "assets/key1",
          &Uri::from_str("https://localhost:8080").unwrap()
        )
        .unwrap(),
      Uri::from_str("https://localhost:8080/assets/key1").unwrap()
    );
  }

  #[tokio::test]
  async fn send_request() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let headers = test_headers(&mut headers);

      let response = String::from_utf8(
        to_bytes(
          storage
            .send_request(
              "assets/key1",
              headers,
              Method::GET,
              &Uri::from_str(&url).unwrap(),
            )
            .await
            .unwrap()
            .into_body(),
        )
        .await
        .unwrap()
        .to_vec(),
      )
      .unwrap();
      assert_eq!(response, "value1");
    })
    .await;
  }

  #[tokio::test]
  async fn get_key() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let headers = test_headers(&mut headers);

      let response = String::from_utf8(
        to_bytes(
          storage
            .get_header("assets/key1", headers)
            .await
            .unwrap()
            .into_body(),
        )
        .await
        .unwrap()
        .to_vec(),
      )
      .unwrap();
      assert_eq!(response, "value1");
    })
    .await;
  }

  #[tokio::test]
  async fn head_key() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let headers = test_headers(&mut headers);

      let response: u64 = storage
        .get_header("assets/key1", headers)
        .await
        .unwrap()
        .headers()
        .get(CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
      assert_eq!(response, 6);
    })
    .await;
  }

  #[tokio::test]
  async fn get_storage() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let headers = test_headers(&mut headers);
      let options = GetOptions::new_with_default_range(headers);

      let mut reader = storage.get("assets/key1", options).await.unwrap();

      let mut response = [0; 6];
      reader.read_exact(&mut response).await.unwrap();

      assert_eq!(String::from_utf8(response.to_vec()).unwrap(), "value1");
    })
    .await;
  }

  #[tokio::test]
  async fn range_url_storage() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let options = test_range_options(&mut headers);

      assert_eq!(
        storage.range_url("assets/key1", options).await.unwrap(),
        HtsGetUrl::new(format!("{}/assets/key1", url))
          .with_headers(Headers::default().with_header(AUTHORIZATION.as_str(), "secret"))
      );
    })
    .await;
  }

  #[tokio::test]
  async fn head_storage() {
    with_url_test_server(|url| async move {
      let storage = UrlStorage::new(
        test_client(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        Uri::from_str(&url).unwrap(),
        true,
        #[cfg(feature = "crypt4gh")]
        None,
      );

      let mut headers = HeaderMap::default();
      let headers = test_headers(&mut headers);
      let options = HeadOptions::new(headers);

      assert_eq!(storage.head("assets/key1", options).await.unwrap(), 6);
    })
    .await;
  }

  #[test]
  fn format_url() {
    let storage = UrlStorage::new(
      test_client(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://localhost:8080").unwrap(),
      true,
      #[cfg(feature = "crypt4gh")]
      None,
    );

    let mut headers = HeaderMap::default();
    let options = test_range_options(&mut headers);

    assert_eq!(
      storage
        .format_url(
          "assets/key1",
          options,
          &Uri::from_str("https://example.com").unwrap()
        )
        .unwrap(),
      HtsGetUrl::new("https://localhost:8080/assets/key1")
        .with_headers(Headers::default().with_header(AUTHORIZATION.as_str(), "secret"))
    );
  }

  #[test]
  fn format_url_different_response_scheme() {
    let storage = UrlStorage::new(
      test_client(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("http://example.com").unwrap(),
      true,
      #[cfg(feature = "crypt4gh")]
      None,
    );

    let mut headers = HeaderMap::default();
    let options = test_range_options(&mut headers);

    assert_eq!(
      storage
        .format_url(
          "assets/key1",
          options,
          &Uri::from_str("https://example.com").unwrap()
        )
        .unwrap(),
      HtsGetUrl::new("http://example.com/assets/key1")
        .with_headers(Headers::default().with_header(AUTHORIZATION.as_str(), "secret"))
    );
  }

  #[test]
  fn format_url_no_headers() {
    let storage = UrlStorage::new(
      test_client(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://localhost:8081").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      Uri::from_str("https://example.com").unwrap(),
      false,
      #[cfg(feature = "crypt4gh")]
      None,
    );

    let mut headers = HeaderMap::default();
    let options = test_range_options(&mut headers);

    assert_eq!(
      storage
        .format_url(
          "assets/key1",
          options,
          &Uri::from_str("https://example.com").unwrap()
        )
        .unwrap(),
      HtsGetUrl::new("https://localhost:8081/assets/key1")
    );
  }

  fn test_client() -> Client<HttpsConnector<HttpConnector>> {
    Client::builder().build(
      HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build(),
    )
  }

  pub(crate) async fn with_url_test_server<F, Fut>(test: F)
  where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = ()>,
  {
    let (_, base_path) = create_local_test_files().await;
    with_test_server(base_path.path(), test).await;
  }

  async fn test_auth<B>(
    request: Request<B>,
    next: Next<B>,
  ) -> result::Result<Response, StatusCode> {
    let auth_header = request
      .headers()
      .get(AUTHORIZATION)
      .and_then(|header| header.to_str().ok());

    match auth_header {
      Some("secret") => Ok(next.run(request).await),
      _ => Err(StatusCode::UNAUTHORIZED),
    }
  }

  pub(crate) async fn with_test_server<F, Fut>(server_base_path: &Path, test: F)
  where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = ()>,
  {
    let router = Router::new()
      .nest_service("/assets", ServeDir::new(server_base_path.to_str().unwrap()))
      .route_layer(middleware::from_fn(test_auth));

    // TODO fix this in htsget-test to bind and return tcp listener.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(
      axum::Server::from_tcp(listener)
        .unwrap()
        .serve(router.into_make_service()),
    );

    test(format!("http://{}", addr)).await;
  }

  fn test_headers(headers: &mut HeaderMap) -> &HeaderMap {
    headers.append(
      HeaderName::from_str(AUTHORIZATION.as_str()).unwrap(),
      HeaderValue::from_str("secret").unwrap(),
    );
    headers
  }

  fn test_range_options(headers: &mut HeaderMap) -> RangeUrlOptions {
    let headers = test_headers(headers);
    let options = RangeUrlOptions::new_with_default_range(headers);

    options
  }
}
