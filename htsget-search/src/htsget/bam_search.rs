//! Module providing the search capability using BAM/BAI files
//!

use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use noodles::bam;
use noodles::bam::bai;
use noodles::bgzf;
use noodles::bgzf::VirtualPosition;
use noodles::csi::index::ReferenceSequence;
use noodles::csi::Index;
use noodles::sam::header::record::value::map::read_group::platform::ParseError;
use noodles::sam::header::record::value::map::read_group::Platform;
use noodles::sam::Header;
use tokio::io;
use tokio::io::{AsyncRead, BufReader};
use tracing::{instrument, trace, warn};

use crate::htsget::search::{BgzfSearch, Search, SearchAll, SearchReads};
use crate::htsget::HtsGetError;
use crate::htsget::ParsedHeader;
use crate::Class::Body;
use crate::{
  htsget::{Format, Query, Result},
  storage::{BytesPosition, Storage},
};

type AsyncReader<ReaderType> = bam::AsyncReader<bgzf::AsyncReader<ReaderType>>;

/// Allows searching through bam files.
pub struct BamSearch<S> {
  storage: Arc<S>,
}

#[async_trait]
impl<S, ReaderType> BgzfSearch<S, ReaderType, AsyncReader<ReaderType>, Header> for BamSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  #[instrument(level = "trace", skip(self, index))]
  async fn get_byte_ranges_for_unmapped(
    &self,
    query: &Query,
    index: &Index,
  ) -> Result<Vec<BytesPosition>> {
    trace!("getting byte ranges for unmapped reads");
    let last_interval = index.first_record_in_last_linear_bin_start_position();
    let start = match last_interval {
      Some(start) => start,
      None => {
        VirtualPosition::try_from((self.get_header_end_offset(index).await?, 0)).map_err(|err| {
          HtsGetError::InvalidInput(format!(
            "invalid virtual position generated from header end offset {err}."
          ))
        })?
      }
    };

    Ok(vec![BytesPosition::default()
      .with_start(start.compressed())
      .with_end(self.position_at_eof(query).await?)
      .with_class(Body)])
  }
}

#[async_trait]
impl<S, ReaderType> Search<S, ReaderType, ReferenceSequence, Index, AsyncReader<ReaderType>, Header>
  for BamSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  fn init_reader(inner: ReaderType) -> AsyncReader<ReaderType> {
    AsyncReader::new(inner)
  }

  async fn read_header(reader: &mut AsyncReader<ReaderType>) -> io::Result<Header> {
    let header = reader.read_header().await;
    reader.read_reference_sequences().await?;

    if let Ok(header) = header.as_deref() {
      for value in header.split_whitespace() {
        if let Some(value) = value.strip_prefix("PL:") {
          if let Err(ParseError::Invalid) = Platform::from_str(value) {
            warn!(
              "invalid read group platform `{value}`, only `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, \
              `{}`, `{}`, `{}`, `{}`, or `{}` is supported",
              Platform::Capillary.as_ref(),
              Platform::DnbSeq.as_ref(),
              Platform::Element.as_ref(),
              Platform::Ls454.as_ref(),
              Platform::Illumina.as_ref(),
              Platform::Solid.as_ref(),
              Platform::Helicos.as_ref(),
              Platform::IonTorrent.as_ref(),
              Platform::Ont.as_ref(),
              Platform::PacBio.as_ref(),
              Platform::Ultima.as_ref()
            );
          }
        }
      }
    }

    Ok(header?.parse::<ParsedHeader<Header>>()?.into_inner())
  }

  async fn read_index_inner<T: AsyncRead + Unpin + Send>(inner: T) -> io::Result<Index> {
    let mut reader = bai::AsyncReader::new(BufReader::new(inner));
    reader.read_header().await?;
    reader.read_index().await
  }

  #[instrument(level = "trace", skip(self, index, header, query))]
  async fn get_byte_ranges_for_reference_name(
    &self,
    reference_name: String,
    index: &Index,
    header: &Header,
    query: &Query,
  ) -> Result<Vec<BytesPosition>> {
    trace!("getting byte ranges for reference name");
    self
      .get_byte_ranges_for_reference_name_reads(&reference_name, index, header, query)
      .await
  }

  fn get_storage(&self) -> Arc<S> {
    Arc::clone(&self.storage)
  }

  fn get_format(&self) -> Format {
    Format::Bam
  }
}

#[async_trait]
impl<S, ReaderType>
  SearchReads<S, ReaderType, ReferenceSequence, Index, AsyncReader<ReaderType>, Header>
  for BamSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  async fn get_reference_sequence_from_name<'a>(
    &self,
    header: &'a Header,
    name: &str,
  ) -> Option<usize> {
    Some(header.reference_sequences().get_index_of(name)?)
  }

  async fn get_byte_ranges_for_unmapped_reads(
    &self,
    query: &Query,
    bai_index: &Index,
  ) -> Result<Vec<BytesPosition>> {
    self.get_byte_ranges_for_unmapped(query, bai_index).await
  }

  async fn get_byte_ranges_for_reference_sequence(
    &self,
    ref_seq_id: usize,
    query: &Query,
    index: &Index,
  ) -> Result<Vec<BytesPosition>> {
    self
      .get_byte_ranges_for_reference_sequence_bgzf(query, ref_seq_id, index)
      .await
  }
}

impl<S, ReaderType> BamSearch<S>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
{
  /// Create the bam search.
  pub fn new(storage: Arc<S>) -> Self {
    Self { storage }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use std::future::Future;

  use htsget_config::storage::local::LocalStorage as ConfigLocalStorage;
  use htsget_test::util::expected_bgzf_eof_data_url;

  #[cfg(feature = "s3-storage")]
  use crate::htsget::from_storage::tests::with_aws_storage_fn;
  use crate::htsget::from_storage::tests::with_local_storage_fn;
  use crate::storage::local::LocalStorage;
  use crate::{Class::Body, Class::Header, Headers, HtsGetError::NotFound, Response, Url};

  use super::*;

  const DATA_LOCATION: &str = "data/bam";
  const INDEX_FILE_LOCATION: &str = "htsnexus_test_NA12878.bam.bai";

  #[tokio::test]
  async fn search_all_reads() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam);
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-2596770")),
          Url::new(expected_bgzf_eof_data_url()),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_unmapped_reads() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
        .with_reference_name("*");
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
            .with_class(Header),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=2060795-2596770"))
            .with_class(Body),
          Url::new(expected_bgzf_eof_data_url()).with_class(Body),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_without_seq_range() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
        .with_reference_name("20");
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
            .with_class(Header),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=977196-2128165"))
            .with_class(Body),
          Url::new(expected_bgzf_eof_data_url()).with_class(Body),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_with_seq_range() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
        .with_reference_name("11")
        .with_start(5015000)
        .with_end(5050000);
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
            .with_class(Header),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=256721-647345"))
            .with_class(Body),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=824361-842100"))
            .with_class(Body),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=977196-996014"))
            .with_class(Body),
          Url::new(expected_bgzf_eof_data_url()).with_class(Body),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_reference_name_no_end_position() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
        .with_reference_name("11")
        .with_start(5015000);
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
            .with_class(Header),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=256721-996014"))
            .with_class(Body),
          Url::new(expected_bgzf_eof_data_url()).with_class(Body),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_many_response_urls() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
        .with_reference_name("11")
        .with_start(4999976)
        .with_end(5003981);
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=0-273085")),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=499249-574358")),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=627987-647345")),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=824361-842100")),
          Url::new(expected_url())
            .with_headers(Headers::default().with_header("Range", "bytes=977196-996014")),
          Url::new(expected_bgzf_eof_data_url()),
        ],
      ));
      assert_eq!(response, expected_response)
    })
    .await
  }

  #[tokio::test]
  async fn search_no_gzi() {
    with_local_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage.clone());
        let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
          .with_reference_name("11")
          .with_start(5015000)
          .with_end(5050000);
        let response = search.search(query).await;
        println!("{response:#?}");

        let expected_response = Ok(Response::new(
          Format::Bam,
          vec![
            Url::new(expected_url())
              .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
              .with_class(Header),
            Url::new(expected_url())
              .with_headers(Headers::default().with_header("Range", "bytes=256721-1065951"))
              .with_class(Body),
            Url::new(expected_bgzf_eof_data_url()).with_class(Body),
          ],
        ));
        assert_eq!(response, expected_response)
      },
      DATA_LOCATION,
      &["htsnexus_test_NA12878.bam", INDEX_FILE_LOCATION],
    )
    .await
  }

  #[tokio::test]
  async fn search_header() {
    with_local_storage(|storage| async move {
      let search = BamSearch::new(storage.clone());
      let query =
        Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam).with_class(Header);
      let response = search.search(query).await;
      println!("{response:#?}");

      let expected_response = Ok(Response::new(
        Format::Bam,
        vec![Url::new(expected_url())
          .with_headers(Headers::default().with_header("Range", "bytes=0-4667"))
          .with_class(Header)],
      ));
      assert_eq!(response, expected_response)
    })
    .await;
  }

  #[tokio::test]
  async fn search_non_existent_id_reference_name() {
    with_local_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage.clone());
        let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam);
        let response = search.search(query).await;
        assert!(matches!(response, Err(NotFound(_))));
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  #[tokio::test]
  async fn search_non_existent_id_all_reads() {
    with_local_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage.clone());
        let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
          .with_reference_name("20");
        let response = search.search(query).await;
        assert!(matches!(response, Err(NotFound(_))));
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  #[tokio::test]
  async fn search_non_existent_id_header() {
    with_local_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage.clone());
        let query =
          Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam).with_class(Header);
        let response = search.search(query).await;
        assert!(matches!(response, Err(NotFound(_))));
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  #[cfg(feature = "s3-storage")]
  #[tokio::test]
  async fn search_non_existent_id_reference_name_aws() {
    with_aws_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage);
        let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam);
        let response = search.search(query).await;
        assert!(response.is_err());
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  #[cfg(feature = "s3-storage")]
  #[tokio::test]
  async fn search_non_existent_id_all_reads_aws() {
    with_aws_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage);
        let query = Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam)
          .with_reference_name("20");
        let response = search.search(query).await;
        assert!(response.is_err());
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  #[cfg(feature = "s3-storage")]
  #[tokio::test]
  async fn search_non_existent_id_header_aws() {
    with_aws_storage_fn(
      |storage| async move {
        let search = BamSearch::new(storage);
        let query =
          Query::new_with_default_request("htsnexus_test_NA12878", Format::Bam).with_class(Header);
        let response = search.search(query).await;
        assert!(response.is_err());
      },
      DATA_LOCATION,
      &[INDEX_FILE_LOCATION],
    )
    .await
  }

  pub(crate) async fn with_local_storage<F, Fut>(test: F)
  where
    F: FnOnce(Arc<LocalStorage<ConfigLocalStorage>>) -> Fut,
    Fut: Future<Output = ()>,
  {
    with_local_storage_fn(test, DATA_LOCATION, &[]).await
  }

  pub(crate) fn expected_url() -> String {
    "http://127.0.0.1:8081/data/htsnexus_test_NA12878.bam".to_string()
  }
}
