//! This module provides search capabilities for CRAM files.
//!

use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;

use async_trait::async_trait;
use futures::prelude::stream::FuturesUnordered;
use futures::StreamExt;
use noodles::cram::crai;
use noodles::cram::crai::{Index, Record};
use noodles::sam;
use noodles::sam::Header;
use noodles_cram::AsyncReader;
use tokio::io::{AsyncRead, AsyncSeek};
use tokio::{io, select};

use crate::htsget::search::{Search, SearchAll, SearchReads};
use crate::htsget::{Format, HtsGetError, Query, Result};
use crate::storage::{BytesRange, Storage};

pub(crate) struct CramSearch<S> {
  storage: Arc<S>,
}

#[async_trait]
impl<S, ReaderType>
  SearchAll<S, ReaderType, PhantomData<Self>, Index, AsyncReader<ReaderType>, Header>
  for CramSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + AsyncSeek + Unpin + Send + Sync,
{
  async fn get_byte_ranges_for_all(
    &self,
    id: String,
    format: Format,
    index: &Index,
  ) -> Result<Vec<BytesRange>> {
    Self::bytes_ranges_from_index(
      self,
      &id,
      &format,
      None,
      Range::default(),
      index,
      Arc::new(|_: &Record| true),
    )
    .await
  }

  async fn get_byte_ranges_for_header(&self, query: &Query) -> Result<Vec<BytesRange>> {
    let (mut reader, _) = self.create_reader(&query.id, &self.get_format()).await?;
    Ok(vec![BytesRange::default()
      .with_start(Self::FILE_DEFINITION_LENGTH)
      .with_end(reader.position().await?)])
  }
}

#[async_trait]
impl<S, ReaderType>
  SearchReads<S, ReaderType, PhantomData<Self>, Index, AsyncReader<ReaderType>, Header>
  for CramSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + AsyncSeek + Unpin + Send + Sync,
{
  async fn get_reference_sequence_from_name<'a>(
    &self,
    header: &'a Header,
    name: &str,
  ) -> Option<(usize, &'a String, &'a sam::header::ReferenceSequence)> {
    header.reference_sequences().get_full(name)
  }

  async fn get_byte_ranges_for_unmapped_reads(
    &self,
    query: &Query,
    index: &Index,
  ) -> Result<Vec<BytesRange>> {
    Self::bytes_ranges_from_index(
      self,
      &query.id,
      &self.get_format(),
      None,
      Range::default(),
      index,
      Arc::new(|record: &Record| record.reference_sequence_id().is_none()),
    )
    .await
  }

  async fn get_byte_ranges_for_reference_sequence(
    &self,
    ref_seq: &sam::header::ReferenceSequence,
    ref_seq_id: usize,
    query: Query,
    index: &Index,
  ) -> Result<Vec<BytesRange>> {
    Self::bytes_ranges_from_index(
      self,
      &query.id,
      &self.get_format(),
      Some(ref_seq),
      query
        .start
        .map(|start| start as i32)
        .unwrap_or(Self::MIN_SEQ_POSITION as i32)
        ..query.end.map(|end| end as i32).unwrap_or(ref_seq.len()),
      index,
      Arc::new(move |record: &Record| record.reference_sequence_id() == Some(ref_seq_id)),
    )
    .await
  }
}

/// PhantomData is used because of a lack of reference sequence data for CRAM.
#[async_trait]
impl<S, ReaderType> Search<S, ReaderType, PhantomData<Self>, Index, AsyncReader<ReaderType>, Header>
  for CramSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + AsyncSeek + Unpin + Send + Sync,
{
  fn init_reader(inner: ReaderType) -> AsyncReader<ReaderType> {
    AsyncReader::new(inner)
  }

  async fn read_raw_header(reader: &mut AsyncReader<ReaderType>) -> io::Result<String> {
    reader.read_file_definition().await?;
    reader.read_file_header().await
  }

  async fn read_index_inner<T: AsyncRead + Send + Unpin>(inner: T) -> io::Result<Index> {
    crai::AsyncReader::new(inner).read_index().await
  }

  async fn get_byte_ranges_for_reference_name(
    &self,
    reference_name: String,
    index: &Index,
    query: Query,
  ) -> Result<Vec<BytesRange>> {
    self
      .get_byte_ranges_for_reference_name_reads(&reference_name, index, query)
      .await
  }

  fn get_storage(&self) -> Arc<S> {
    self.storage.clone()
  }

  fn get_format(&self) -> Format {
    Format::Cram
  }
}

impl<S, ReaderType> CramSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + AsyncSeek + Unpin + Send + Sync,
{
  const FILE_DEFINITION_LENGTH: u64 = 26;
  const EOF_CONTAINER_LENGTH: u64 = 38;

  pub fn new(storage: Arc<S>) -> Self {
    Self { storage }
  }

  /// Get bytes ranges using the index.
  async fn bytes_ranges_from_index<F>(
    &self,
    id: &str,
    format: &Format,
    ref_seq: Option<&sam::header::ReferenceSequence>,
    seq_range: Range<i32>,
    crai_index: &[crai::Record],
    predicate: Arc<F>,
  ) -> Result<Vec<BytesRange>>
  where
    F: Fn(&Record) -> bool + Send + Sync + 'static,
  {
    // This could be improved by using some sort of index mapping.
    let mut futures = FuturesUnordered::new();
    for (record, next) in crai_index.iter().zip(crai_index.iter().skip(1)) {
      let owned_record = record.clone();
      let owned_next = next.clone();
      let ref_seq_owned = ref_seq.cloned();
      let owned_predicate = predicate.clone();
      let range = seq_range.clone();
      futures.push(tokio::spawn(async move {
        if owned_predicate(&owned_record) {
          Self::bytes_ranges_for_record(ref_seq_owned.as_ref(), range, &owned_record, &owned_next)
        } else {
          None
        }
      }));
    }

    let mut byte_ranges = Vec::new();
    loop {
      select! {
        Some(next) = futures.next() => {
          if let Some(range) = next.map_err(HtsGetError::from)? {
            byte_ranges.push(range);
          }
        },
        else => break
      }
    }

    let last = crai_index
      .last()
      .ok_or_else(|| HtsGetError::invalid_input("No entries in CRAI"))?;
    if predicate(last) {
      let file_size = self
        .storage
        .head(format.fmt_file(id))
        .await
        .map_err(|_| HtsGetError::io_error("Reading CRAM file size."))?;
      let eof_position = file_size - Self::EOF_CONTAINER_LENGTH;
      byte_ranges.push(
        BytesRange::default()
          .with_start(last.offset())
          .with_end(eof_position),
      );
    }

    Ok(BytesRange::merge_all(byte_ranges))
  }

  /// Gets bytes ranges for a specific index entry.
  pub(crate) fn bytes_ranges_for_record(
    ref_seq: Option<&sam::header::ReferenceSequence>,
    seq_range: Range<i32>,
    record: &Record,
    next: &Record,
  ) -> Option<BytesRange> {
    match ref_seq {
      None => Some(
        BytesRange::default()
          .with_start(record.offset())
          .with_end(next.offset()),
      ),
      Some(_) => {
        let start = record
          .alignment_start()
          .map(usize::from)
          .unwrap_or_default() as i32;
        if seq_range.start <= start + record.alignment_span() as i32 && seq_range.end >= start {
          Some(
            BytesRange::default()
              .with_start(record.offset())
              .with_end(next.offset()),
          )
        } else {
          None
        }
      }
    }
  }
}

#[cfg(test)]
pub mod tests {
  use std::future::Future;

  use htsget_config::regex_resolver::RegexResolver;

  use crate::htsget::{Class, Headers, Response, Url};
  use crate::storage::axum_server::HttpsFormatter;
  use crate::storage::local::LocalStorage;

  use super::*;

  #[tokio::test]
  async fn search_all_reads() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=6087-1627756"))],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_unmapped_reads() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram).with_reference_name("*");
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=1280106-1627756"))],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_without_seq_range() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram).with_reference_name("20");
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=604231-1280106"))],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_with_seq_range_no_overlap() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram)
        .with_reference_name("11")
        .with_start(5000000)
        .with_end(5050000);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=6087-465709"))],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_with_seq_range_overlap() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram)
        .with_reference_name("11")
        .with_start(5000000)
        .with_end(5100000);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=6087-604231"))],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_header() {
    with_local_storage(|storage| async move {
      let search = CramSearch::new(storage.clone());
      let query = Query::new("htsnexus_test_NA12878", Format::Cram).with_class(Class::Header);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Cram,
        vec![Url::new(expected_url(storage))
          .with_headers(Headers::default().with_header("Range", "bytes=26-6087"))
          .with_class(Class::Header)],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  pub(crate) async fn with_local_storage<F, Fut>(test: F)
  where
    F: FnOnce(Arc<LocalStorage<HttpsFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    let base_path = std::env::current_dir()
      .unwrap()
      .parent()
      .unwrap()
      .join("data/cram");
    test(Arc::new(
      LocalStorage::new(
        base_path,
        RegexResolver::new(".*", "$0").unwrap(),
        HttpsFormatter::new("127.0.0.1", "8081").unwrap(),
      )
      .unwrap(),
    ))
    .await
  }

  pub(crate) fn expected_url(storage: Arc<LocalStorage<HttpsFormatter>>) -> String {
    format!(
      "https://127.0.0.1:8081{}",
      storage
        .base_path()
        .join("htsnexus_test_NA12878.cram")
        .to_string_lossy()
    )
  }
}
