//! Module providing the search capability using VCF files
//!

use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::stream::FuturesOrdered;
use noodles::bgzf;
use noodles::csi::index::reference_sequence::bin::Chunk;
use noodles::csi::BinningIndex;
use noodles::tabix;
use noodles::tabix::index::ReferenceSequence;
use noodles::tabix::Index;
use noodles::vcf::Header;
use noodles_vcf as vcf;
use tokio::io;
use tokio::io::AsyncRead;

use crate::htsget::search::{find_first, BgzfSearch, BinningIndexExt, Search};
use crate::htsget::HtsGetError;
use crate::{
  htsget::{Format, Query, Result},
  storage::{BytesPosition, Storage},
};

type AsyncReader<ReaderType> = vcf::AsyncReader<bgzf::AsyncReader<ReaderType>>;

pub(crate) struct VcfSearch<S> {
  storage: Arc<S>,
}

impl BinningIndexExt for Index {
  fn get_all_chunks(&self) -> Vec<&Chunk> {
    self
      .reference_sequences()
      .iter()
      .flat_map(|ref_seq| ref_seq.bins())
      .flat_map(|bin| bin.chunks())
      .collect()
  }
}

#[async_trait]
impl<S, ReaderType>
  BgzfSearch<S, ReaderType, ReferenceSequence, Index, AsyncReader<ReaderType>, Header>
  for VcfSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  type ReferenceSequenceHeader = PhantomData<Self>;

  fn max_seq_position(_ref_seq: &Self::ReferenceSequenceHeader) -> usize {
    Self::MAX_SEQ_POSITION
  }
}

#[async_trait]
impl<S, ReaderType> Search<S, ReaderType, ReferenceSequence, Index, AsyncReader<ReaderType>, Header>
  for VcfSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  fn init_reader(inner: ReaderType) -> AsyncReader<ReaderType> {
    AsyncReader::new(bgzf::AsyncReader::new(inner))
  }

  async fn read_raw_header(reader: &mut AsyncReader<ReaderType>) -> io::Result<String> {
    reader.read_header().await
  }

  async fn read_index_inner<T: AsyncRead + Unpin + Send>(inner: T) -> io::Result<Index> {
    tabix::AsyncReader::new(inner).read_index().await
  }

  async fn get_byte_ranges_for_reference_name(
    &self,
    reference_name: String,
    index: &Index,
    header: &Header,
    mut query: Query,
  ) -> Result<Vec<BytesPosition>> {
    let maybe_len = header
      .contigs()
      .get(&reference_name)
      .and_then(|contig| contig.len());

    // We are assuming the order of the names and the references sequences
    // in the index is the same
    let mut futures = FuturesOrdered::new();
    for (index, name) in index.header().reference_sequence_names().iter().enumerate() {
      let owned_name = name.to_owned();
      let owned_reference_name = reference_name.clone();
      futures.push(tokio::spawn(async move {
        if owned_name == owned_reference_name {
          Some(index)
        } else {
          None
        }
      }));
    }
    let ref_seq_index = find_first(
      &format!(
        "Reference name not found in the TBI file: {}",
        reference_name,
      ),
      futures,
    )
    .await?;

    query.interval.end = match query.interval.end {
      None => maybe_len
        .map(u32::try_from)
        .transpose()
        .map_err(|_| HtsGetError::invalid_input("Failed to convert contig length to u32."))?,
      value => value,
    };

    let byte_ranges = self
      .get_byte_ranges_for_reference_sequence_bgzf(query, &PhantomData, ref_seq_index, index)
      .await?;
    Ok(byte_ranges)
  }

  fn get_storage(&self) -> Arc<S> {
    self.storage.clone()
  }

  fn get_format(&self) -> Format {
    Format::Vcf
  }
}

impl<S, ReaderType> VcfSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  // 1-based
  pub(crate) const MAX_SEQ_POSITION: usize = (1 << 29) - 1; // see https://github.com/zaeleus/noodles/issues/25#issuecomment-868871298

  /// Create the vcf search.
  pub fn new(storage: Arc<S>) -> Self {
    Self { storage }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use std::future::Future;

  use htsget_test_utils::util::expected_bgzf_eof_data_url;

  use crate::htsget::from_storage::tests::{
    with_local_storage as with_local_storage_path,
    with_local_storage_tmp as with_local_storage_tmp_path,
  };
  use crate::htsget::{Class, Class::Body, Headers, Response, Url};
  use crate::storage::local::LocalStorage;
  use crate::storage::ticket_server::HttpTicketFormatter;

  use super::*;

  #[tokio::test]
  async fn search_all_variants() {
    with_local_storage(|storage| async move {
      let search = VcfSearch::new(storage.clone());
      let filename = "sample1-bcbio-cancer";
      let query = Query::new(filename, Format::Vcf);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(expected_vcf_response(filename));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_without_seq_range() {
    with_local_storage(|storage| async move {
      let search = VcfSearch::new(storage.clone());
      let filename = "spec-v4.3";
      let query = Query::new(filename, Format::Vcf).with_reference_name("20");
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Vcf,
        vec![
          Url::new(expected_url(filename))
            .with_headers(Headers::default().with_header("Range", "bytes=0-822")),
          Url::new(expected_bgzf_eof_data_url()).with_class(Body),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_with_seq_range() {
    with_local_storage(|storage| async move { test_reference_name_with_seq_range(storage).await })
      .await;
  }

  #[tokio::test]
  async fn search_no_gzi() {
    with_local_storage_tmp(
      |storage| async move { test_reference_name_with_seq_range(storage).await },
    )
    .await;
  }

  #[tokio::test]
  async fn search_header() {
    with_local_storage(|storage| async move {
      let search = VcfSearch::new(storage.clone());
      let filename = "spec-v4.3";
      let query = Query::new(filename, Format::Vcf).with_class(Class::Header);
      let response = search.search(query).await;
      println!("{:#?}", response);

      let expected_response = Ok(Response::new(
        Format::Vcf,
        vec![Url::new(expected_url(filename))
          .with_headers(Headers::default().with_header("Range", "bytes=0-822"))
          .with_class(Class::Header)],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  async fn test_reference_name_with_seq_range(storage: Arc<LocalStorage<HttpTicketFormatter>>) {
    let search = VcfSearch::new(storage.clone());
    let filename = "sample1-bcbio-cancer";
    let query = Query::new(filename, Format::Vcf)
      .with_reference_name("chrM")
      .with_start(151)
      .with_end(153);
    let response = search.search(query).await;
    println!("{:#?}", response);

    let expected_response = Ok(expected_vcf_response(filename));
    assert_eq!(response, expected_response);
  }

  fn expected_vcf_response(filename: &str) -> Response {
    Response::new(
      Format::Vcf,
      vec![
        Url::new(expected_url(filename))
          .with_headers(Headers::default().with_header("Range", "bytes=0-3465")),
        Url::new(expected_bgzf_eof_data_url()).with_class(Body),
      ],
    )
  }

  pub(crate) async fn with_local_storage<F, Fut>(test: F)
  where
    F: FnOnce(Arc<LocalStorage<HttpTicketFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    with_local_storage_path(test, "data/vcf").await
  }

  async fn with_local_storage_tmp<F, Fut>(test: F)
  where
    F: FnOnce(Arc<LocalStorage<HttpTicketFormatter>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    with_local_storage_tmp_path(
      test,
      "data/vcf",
      &[
        "sample1-bcbio-cancer.vcf.gz",
        "sample1-bcbio-cancer.vcf.gz.tbi",
      ],
    )
    .await
  }

  pub(crate) fn expected_url(name: &str) -> String {
    format!("http://127.0.0.1:8081/data/{}.vcf.gz", name)
  }
}
