use crypt4gh::error::Crypt4GHError;
use std::array::TryFromSliceError;
use std::num::TryFromIntError;
use std::{io, result};
use thiserror::Error;
use tokio::task;

/// The result type for Crypt4GH errors.
pub type Result<T> = result::Result<T, Error>;

/// Errors related to Crypt4GH.
#[derive(Error, Debug)]
pub enum Error {
  #[error("converting slice to fixed size array")]
  SliceConversionError,
  #[error("converting between numeric types")]
  NumericConversionError,
  #[error("decoding header info: `{0}`")]
  DecodingHeaderInfo(Crypt4GHError),
  #[error("decoding header packet: `{0}`")]
  DecodingHeaderPacket(Crypt4GHError),
  #[error("io error: `{0}`")]
  IOError(io::Error),
  #[error("join handle error: `{0}`")]
  JoinHandleError(task::JoinError),
  #[error("maximum header size exceeded")]
  MaximumHeaderSize,
}

impl From<io::Error> for Error {
  fn from(error: io::Error) -> Self {
    Self::IOError(error)
  }
}