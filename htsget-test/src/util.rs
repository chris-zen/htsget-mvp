use std::fs;
use std::path::{Path, PathBuf};

use rcgen::generate_simple_self_signed;

pub fn generate_test_certificates<P: AsRef<Path>>(
  in_path: P,
  key_name: &str,
  cert_name: &str,
) -> (PathBuf, PathBuf) {
  let key_path = in_path.as_ref().join(key_name);
  let cert_path = in_path.as_ref().join(cert_name);

  let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
  fs::write(&key_path, cert.serialize_private_key_pem()).unwrap();
  fs::write(&cert_path, cert.serialize_pem().unwrap()).unwrap();

  (key_path, cert_path)
}

pub fn expected_bgzf_eof_data_url() -> String {
  "data:;base64,H4sIBAAAAAAA/wYAQkMCABsAAwAAAAAAAAAAAA==".to_string()
}

pub fn expected_cram_eof_data_url() -> String {
  "data:;base64,DwAAAP////8P4EVPRgAAAAABAAW92U8AAQAGBgEAAQABAO5jAUs=".to_string()
}
