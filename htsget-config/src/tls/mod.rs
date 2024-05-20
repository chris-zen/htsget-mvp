//! Configuration related to TLS.
//!

#[cfg(feature = "url-storage")]
pub mod client;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::read_one;
use serde::{Deserialize, Serialize};

use crate::error::Error::{IoError, ParseError};
use crate::error::{Error, Result};
use crate::types::Scheme;
use crate::types::Scheme::{Http, Https};

/// A trait to determine which scheme a key pair option has.
pub trait KeyPairScheme {
  /// Get the scheme.
  fn get_scheme(&self) -> Scheme;
}

/// A certificate and key pair used for TLS. Serialization is not implemented because there
/// is no way to convert back to a `PathBuf`.
#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "CertificateKeyPairPath")]
pub struct TlsServerConfig {
  server_config: ServerConfig,
}

impl TlsServerConfig {
  /// Create a new TlsServerConfig.
  pub fn new(server_config: ServerConfig) -> Self {
    Self { server_config }
  }

  /// Get the inner server config.
  pub fn into_inner(self) -> ServerConfig {
    self.server_config
  }
}

/// The location of a certificate and key pair used for TLS.
/// This is the path to the PEM formatted X.509 certificate and private key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CertificateKeyPairPath {
  cert: PathBuf,
  key: PathBuf,
}

/// The certificate and key pair used for TLS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateKeyPair {
  certs: Vec<Certificate>,
  key: PrivateKey,
}

impl CertificateKeyPair {
  /// Create a new CertificateKeyPair.
  pub fn new(certs: Vec<Certificate>, key: PrivateKey) -> Self {
    Self { certs, key }
  }

  /// Get the owned certificate and private key.
  pub fn into_inner(self) -> (Vec<Certificate>, PrivateKey) {
    (self.certs, self.key)
  }
}

/// The location of a certificate and key pair used for TLS.
/// This is the path to the PEM formatted X.509 certificate and private key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RootCertStorePair {
  #[serde(flatten)]
  key_pair: Option<CertificateKeyPairPath>,
  root_store: Option<PathBuf>,
}

impl RootCertStorePair {
  /// Create a new RootCertStorePair.
  pub fn new(key_pair: Option<CertificateKeyPairPath>, root_store: Option<PathBuf>) -> Self {
    Self {
      key_pair,
      root_store,
    }
  }

  /// Get the owned root store pair.
  pub fn into_inner(self) -> (Option<CertificateKeyPairPath>, Option<PathBuf>) {
    (self.key_pair, self.root_store)
  }
}

impl TryFrom<CertificateKeyPairPath> for TlsServerConfig {
  type Error = Error;

  fn try_from(key_pair: CertificateKeyPairPath) -> Result<Self> {
    let server_config = tls_server_config(key_pair.try_into()?)?;

    Ok(Self::new(server_config))
  }
}

impl TryFrom<CertificateKeyPairPath> for CertificateKeyPair {
  type Error = Error;

  fn try_from(key_pair: CertificateKeyPairPath) -> Result<Self> {
    let certs = load_certs(key_pair.cert)?;
    let key = load_key(key_pair.key)?;

    Ok(CertificateKeyPair::new(certs, key))
  }
}

impl CertificateKeyPairPath {
  /// Create a new certificate key pair.
  pub fn new(cert: PathBuf, key: PathBuf) -> Self {
    Self { cert, key }
  }

  /// Get the certs path.
  pub fn certs(&self) -> &Path {
    &self.cert
  }

  /// Get the key path.
  pub fn key(&self) -> &Path {
    &self.key
  }
}

impl KeyPairScheme for Option<&TlsServerConfig> {
  fn get_scheme(&self) -> Scheme {
    match self {
      None => Http,
      Some(_) => Https,
    }
  }
}

/// Load a private key from a file. Supports RSA, PKCS8, and Sec1 encoded keys.
pub fn load_key<P: AsRef<Path>>(key: P) -> Result<PrivateKey> {
  let mut key_reader = BufReader::new(
    File::open(key).map_err(|err| IoError(format!("failed to open key file: {}", err)))?,
  );

  loop {
    match read_one(&mut key_reader)
      .map_err(|err| ParseError(format!("failed to parse private key: {}", err)))?
    {
      Some(rustls_pemfile::Item::RSAKey(key)) => return Ok(PrivateKey(key)),
      Some(rustls_pemfile::Item::PKCS8Key(key)) => return Ok(PrivateKey(key)),
      Some(rustls_pemfile::Item::ECKey(key)) => return Ok(PrivateKey(key)),
      None => break,
      _ => {}
    }
  }

  Err(ParseError("no key found in pem file".to_string()))
}

/// Read byte data.
pub fn read_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  let mut bytes = vec![];
  File::open(path)
    .map_err(|err| IoError(format!("failed to open certificate or key file: {}", err)))?
    .read_to_end(&mut bytes)
    .map_err(|err| IoError(format!("failed to read certificate or key bytes: {}", err)))?;
  Ok(bytes)
}

/// Load certificates from a file.
pub fn load_certs<P: AsRef<Path>>(certs: P) -> Result<Vec<Certificate>> {
  let mut cert_reader = BufReader::new(
    File::open(certs).map_err(|err| IoError(format!("failed to open cert file: {}", err)))?,
  );

  let certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_reader)
    .map_err(|err| ParseError(format!("failed to parse certificates: {}", err)))?
    .into_iter()
    .map(Certificate)
    .collect();

  if certs.is_empty() {
    return Err(ParseError("no certificates found in pem file".to_string()));
  }

  Ok(certs)
}

/// Load TLS server config.
pub fn tls_server_config(key_pair: CertificateKeyPair) -> Result<ServerConfig> {
  let (certs, key) = key_pair.into_inner();

  let mut config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|err| ParseError(err.to_string()))?;

  config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

  Ok(config)
}

#[cfg(test)]
pub(crate) mod tests {
  use std::fs::write;
  use std::io::Cursor;
  use std::path::Path;

  use rcgen::generate_simple_self_signed;
  use rustls_pemfile::{certs, pkcs8_private_keys};
  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_load_key() {
    with_test_certificates(|path, key, _| {
      let key_path = path.join("key.pem");
      let loaded_key = load_key(key_path).unwrap();

      assert_eq!(loaded_key, key);
    });
  }

  #[test]
  fn test_load_cert() {
    with_test_certificates(|path, _, cert| {
      let cert_path = path.join("cert.pem");
      let certs = load_certs(cert_path).unwrap();

      assert_eq!(certs.len(), 1);
      assert_eq!(certs.into_iter().next().unwrap(), cert);
    });
  }

  #[tokio::test]
  async fn test_tls_server_config() {
    with_test_certificates(|_, key, cert| {
      let server_config = tls_server_config(CertificateKeyPair::new(vec![cert], key)).unwrap();

      assert_eq!(
        server_config.alpn_protocols,
        vec![b"h2".to_vec(), b"http/1.1".to_vec()]
      );
    });
  }

  pub(crate) fn with_test_certificates<F>(test: F)
  where
    F: FnOnce(&Path, PrivateKey, Certificate),
  {
    let tmp_dir = TempDir::new().unwrap();

    let key_path = tmp_dir.path().join("key.pem");
    let cert_path = tmp_dir.path().join("cert.pem");

    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();

    let key = cert.serialize_private_key_pem();
    let cert = cert.serialize_pem().unwrap();

    write(key_path, &key).unwrap();
    write(cert_path, &cert).unwrap();

    let key = PrivateKey(
      pkcs8_private_keys(&mut Cursor::new(key))
        .unwrap()
        .into_iter()
        .next()
        .unwrap(),
    );
    let cert = Certificate(
      certs(&mut Cursor::new(cert))
        .unwrap()
        .into_iter()
        .next()
        .unwrap(),
    );

    test(tmp_dir.path(), key, cert);
  }
}
