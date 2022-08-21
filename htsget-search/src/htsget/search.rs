//! The following file defines commonalities between all the file formats. While each format has
//! its own particularities, there are many shared components that can be abstracted.
//!
//! The generic types represent the specifics of the formats, and allow the abstractions to be made,
//! where the names of the types indicate their purpose.
//!

use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures_util::stream::FuturesOrdered;
use noodles::bgzf::gzi;
use noodles::csi::index::reference_sequence::bin::Chunk;
use noodles::csi::{BinningIndex, BinningIndexReferenceSequence};
use noodles::sam;
use tokio::io;
use tokio::io::AsyncRead;
use tokio::select;
use tokio::task::JoinHandle;

use crate::htsget::Class::Body;
use crate::storage::{DataBlock, GetOptions};
use crate::{
  htsget::{Class, Format, HtsGetError, Query, Response, Result},
  storage::{BytesPosition, RangeUrlOptions, Storage},
};

// § 4.1.2 End-of-file marker <https://samtools.github.io/hts-specs/SAMv1.pdf>.
pub(crate) static BGZF_EOF: &[u8] = &[
  0x1f, 0x8b, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x06, 0x00, 0x42, 0x43, 0x02, 0x00,
  0x1b, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Helper function to find the first non-none value from a set of futures.
pub(crate) async fn find_first<T>(
  msg: &str,
  mut futures: FuturesOrdered<JoinHandle<Option<T>>>,
) -> Result<T> {
  let mut result = None;
  loop {
    select! {
      Some(next) = futures.next() => {
        if let Some(next) = next.map_err(HtsGetError::from)? {
          result = Some(next);
          break;
        }
      },
      else => break
    };
  }
  result.ok_or_else(|| HtsGetError::not_found(msg))
}

/// [SearchAll] represents searching bytes ranges that are applicable to all formats. Specifically,
/// range for the whole file, and the header.
///
/// [S] is the storage type.
/// [ReaderType] is the inner type used for [Reader].
/// [ReferenceSequence] is the reference sequence type of the format's index.
/// [Index] is the format's index type.
/// [Reader] is the format's reader type.
/// [Header] is the format's header type.
#[async_trait]
pub(crate) trait SearchAll<S, ReaderType, ReferenceSequence, Index, Reader, Header>
where
  Index: Send + Sync,
{
  /// This returns mapped and placed unmapped ranges.
  async fn get_byte_ranges_for_all(&self, id: String, format: Format)
    -> Result<Vec<BytesPosition>>;

  /// Get the offset in the file of the end of the header.
  async fn get_header_end_offset(&self, index: &Index) -> Result<u64>;

  /// Returns the header bytes range.
  async fn get_byte_ranges_for_header(&self, index: &Index) -> Result<BytesPosition> {
    Ok(
      BytesPosition::default()
        .with_end(self.get_header_end_offset(index).await?)
        .with_class(Class::Header),
    )
  }

  /// Get the eof marker for this format.
  fn get_eof_marker(&self) -> &[u8];

  /// Get the eof data block for this format.
  fn get_eof_data_block(&self) -> Option<DataBlock>;
}

/// [SearchReads] represents searching bytes ranges for the reads endpoint.
///
/// [S] is the storage type.
/// [ReaderType] is the inner type used for [Reader].
/// [ReferenceSequence] is the reference sequence type of the format's index.
/// [Index] is the format's index type.
/// [Reader] is the format's reader type.
/// [Header] is the format's header type.
#[async_trait]
pub(crate) trait SearchReads<S, ReaderType, ReferenceSequence, Index, Reader, Header>:
  Search<S, ReaderType, ReferenceSequence, Index, Reader, Header>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
  Reader: Send,
  Header: FromStr + Send + Sync,
  <Header as FromStr>::Err: Display,
  Index: Send + Sync,
{
  /// Get reference sequence from name.
  async fn get_reference_sequence_from_name<'b>(
    &self,
    header: &'b Header,
    name: &str,
  ) -> Option<(usize, &'b String, &'b sam::header::ReferenceSequence)>;

  /// Get unplaced unmapped ranges.
  async fn get_byte_ranges_for_unmapped_reads(
    &self,
    query: &Query,
    index: &Index,
  ) -> Result<Vec<BytesPosition>>;

  /// Get reads ranges for a reference sequence implementation.
  async fn get_byte_ranges_for_reference_sequence(
    &self,
    reference_sequence: &sam::header::ReferenceSequence,
    ref_seq_id: usize,
    query: Query,
    index: &Index,
  ) -> Result<Vec<BytesPosition>>;

  ///Get reads for a given reference name and an optional sequence range.
  async fn get_byte_ranges_for_reference_name_reads(
    &self,
    reference_name: &str,
    index: &Index,
    header: &Header,
    query: Query,
  ) -> Result<Vec<BytesPosition>> {
    if reference_name == "*" {
      return self.get_byte_ranges_for_unmapped_reads(&query, index).await;
    }

    let maybe_ref_seq = self
      .get_reference_sequence_from_name(header, reference_name)
      .await;

    let byte_ranges = match maybe_ref_seq {
      None => Err(HtsGetError::not_found(format!(
        "reference name not found: {}",
        reference_name
      ))),
      Some((bam_ref_seq_idx, _, bam_ref_seq)) => {
        Self::get_byte_ranges_for_reference_sequence(
          self,
          bam_ref_seq,
          bam_ref_seq_idx,
          query,
          index,
        )
        .await
      }
    }?;
    Ok(byte_ranges)
  }
}

/// [Search] is the general trait that all formats implement, including functions from [SearchAll].
///
/// [S] is the storage type.
/// [ReaderType] is the inner type used for [Reader].
/// [ReferenceSequence] is the reference sequence type of the format's index.
/// [Index] is the format's index type.
/// [Reader] is the format's reader type.
/// [Header] is the format's header type.
#[async_trait]
pub(crate) trait Search<S, ReaderType, ReferenceSequence, Index, Reader, Header>:
  SearchAll<S, ReaderType, ReferenceSequence, Index, Reader, Header>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
  Index: Send + Sync,
  Header: FromStr + Send + Sync,
  <Header as FromStr>::Err: Display,
  Reader: Send,
  Self: Sync + Send,
{
  fn init_reader(inner: ReaderType) -> Reader;
  async fn read_raw_header(reader: &mut Reader) -> io::Result<String>;
  async fn read_index_inner<T: AsyncRead + Unpin + Send>(inner: T) -> io::Result<Index>;

  /// Get ranges for a given reference name and an optional sequence range.
  async fn get_byte_ranges_for_reference_name(
    &self,
    reference_name: String,
    index: &Index,
    header: &Header,
    query: Query,
  ) -> Result<Vec<BytesPosition>>;

  /// Get the storage of this format.
  fn get_storage(&self) -> Arc<S>;

  /// Get the format of this format.
  fn get_format(&self) -> Format;

  /// Get the position at the end of file marker.
  async fn position_at_eof(&self, id: &str, format: &Format) -> Result<u64> {
    let file_size = self.get_storage().head(format.fmt_file(id)).await?;
    Ok(
      file_size
        - u64::try_from(self.get_eof_marker().len())
          .map_err(|err| HtsGetError::InvalidInput(err.to_string()))?,
    )
  }

  /// Read the index from the key.
  async fn read_index(&self, id: &str) -> Result<Index> {
    let storage = self
      .get_storage()
      .get(self.get_format().fmt_index(id), GetOptions::default())
      .await?;
    Self::read_index_inner(storage)
      .await
      .map_err(|err| HtsGetError::io_error(format!("reading {} index: {}", self.get_format(), err)))
  }

  /// Search based on the query.
  async fn search(&self, query: Query) -> Result<Response> {
    match query.class {
      Body => {
        let format = self.get_format();
        if format != query.format {
          return Err(HtsGetError::unsupported_format(format!(
            "using `{}` search, but query contains `{}` format",
            format, query.format
          )));
        }

        let id = query.id.clone();
        let byte_ranges = match query.reference_name.as_ref() {
          None => {
            self
              .get_byte_ranges_for_all(query.id.clone(), format)
              .await?
          }
          Some(reference_name) => {
            let index = self.read_index(&query.id).await?;
            let header = self.get_header(&id, &format, &index).await?;

            let mut byte_ranges = self
              .get_byte_ranges_for_reference_name(reference_name.clone(), &index, &header, query)
              .await?;
            byte_ranges.push(self.get_byte_ranges_for_header(&index).await?);

            byte_ranges
          }
        };

        let mut blocks = DataBlock::from_bytes_positions(byte_ranges);
        if let Some(eof) = self.get_eof_data_block() {
          blocks.push(eof);
        }

        self.build_response(id, format, blocks).await
      }
      Class::Header => {
        let index = self.read_index(&query.id).await?;
        let header_byte_ranges = self.get_byte_ranges_for_header(&index).await?;

        self
          .build_response(
            query.id,
            self.get_format(),
            DataBlock::from_bytes_positions(vec![header_byte_ranges]),
          )
          .await
      }
    }
  }

  /// Build the response from the query using urls.
  async fn build_response(
    &self,
    id: String,
    format: Format,
    byte_ranges: Vec<DataBlock>,
  ) -> Result<Response> {
    let mut storage_futures = FuturesOrdered::new();
    for block in DataBlock::update_classes(byte_ranges) {
      match block {
        DataBlock::Range(range) => {
          let storage = self.get_storage();
          let id = id.clone();
          storage_futures.push_back(tokio::spawn(async move {
            storage
              .range_url(format.fmt_file(&id), RangeUrlOptions::from(range))
              .await
          }));
        }
        DataBlock::Data(data, class) => {
          storage_futures.push_back(tokio::spawn(async move { Ok(S::data_url(data, class)) }));
        }
      }
    }
    let mut urls = Vec::new();
    loop {
      select! {
        Some(next) = storage_futures.next() => urls.push(next.map_err(HtsGetError::from)?.map_err(HtsGetError::from)?),
        else => break
      }
    }
    return Ok(Response::new(format, urls));
  }

  /// Get the header from the file specified by the id and format.
  async fn get_header(&self, id: &str, format: &Format, index: &Index) -> Result<Header> {
    let get_options =
      GetOptions::default().with_range(self.get_byte_ranges_for_header(index).await?);
    let reader_type = self
      .get_storage()
      .get(format.fmt_file(id), get_options)
      .await?;
    let mut reader = Self::init_reader(reader_type);

    Self::read_raw_header(&mut reader)
      .await
      .map_err(|err| {
        HtsGetError::io_error(format!("reading `{}` header: {}", self.get_format(), err))
      })?
      .parse::<Header>()
      .map_err(|err| {
        HtsGetError::parse_error(format!("parsing `{}` header: {}", self.get_format(), err))
      })
  }
}

/// The [BgzfSearch] trait defines commonalities for the formats that use a binning index, specifically
/// BAM, BCF, and VCF.
///
/// [S] is the storage type.
/// [ReaderType] is the inner type used for [Reader].
/// [ReferenceSequence] is the reference sequence type of the format's index.
/// [Index] is the format's index type.
/// [Reader] is the format's reader type.
/// [Header] is the format's header type.
#[async_trait]
pub(crate) trait BgzfSearch<S, ReaderType, ReferenceSequence, Index, Reader, Header>:
  Search<S, ReaderType, ReferenceSequence, Index, Reader, Header>
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
  Reader: Send + Sync,
  ReferenceSequence: BinningIndexReferenceSequence,
  Index: BinningIndex + BinningIndexExt + Send + Sync,
  Header: FromStr + Send + Sync,
  <Header as FromStr>::Err: Display,
{
  type ReferenceSequenceHeader: Sync;

  /// Get the max sequence position.
  fn max_seq_position(ref_seq: &Self::ReferenceSequenceHeader) -> usize;

  fn index_positions(index: &Index) -> Vec<u64> {
    let mut positions = HashSet::new();

    // Its probably most robust to search through all chunks in all reference sequences.
    // See https://github.com/samtools/htslib/issues/1482
    positions.extend(
      index
        .get_all_chunks()
        .iter()
        .flat_map(|chunk| [chunk.start().compressed(), chunk.end().compressed()]),
    );
    positions.extend(
      index
        .reference_sequences()
        .iter()
        .filter_map(|ref_seq| ref_seq.metadata())
        .flat_map(|metadata| {
          [
            metadata.start_position().compressed(),
            metadata.end_position().compressed(),
          ]
        }),
    );

    positions.remove(&0);
    let mut positions: Vec<u64> = positions.into_iter().collect();
    positions.sort_unstable();
    positions
  }

  /// Get ranges for a reference sequence for the bgzf format.
  async fn get_byte_ranges_for_reference_sequence_bgzf(
    &self,
    query: Query,
    reference_sequence: &Self::ReferenceSequenceHeader,
    ref_seq_id: usize,
    index: &Index,
  ) -> Result<Vec<BytesPosition>> {
    let chunks = index
      .query(
        ref_seq_id,
        query
          .interval
          .into_one_based(|| Self::max_seq_position(reference_sequence))?,
      )
      .map_err(|err| HtsGetError::InvalidRange(format!("querying range: {}", err)))?;

    let gzi_data = self
      .get_storage()
      .get(self.get_format().fmt_gzi(&query.id)?, GetOptions::default())
      .await;
    let byte_ranges: Vec<BytesPosition> = match gzi_data {
      Ok(gzi_data) => {
        let gzi = gzi::AsyncReader::new(gzi_data).read_index().await?;
        self
          .bytes_positions_from_chunks(
            chunks,
            &query.id,
            &query.format,
            gzi.into_iter().map(|(compressed, _)| compressed),
          )
          .await?
      }
      Err(_) => {
        self
          .bytes_positions_from_chunks(
            chunks,
            &query.id,
            &query.format,
            Self::index_positions(index).into_iter(),
          )
          .await?
      }
    };

    Ok(byte_ranges)
  }

  async fn bytes_positions_from_chunks<'a>(
    &self,
    chunks: Vec<Chunk>,
    id: &str,
    format: &Format,
    mut positions: impl Iterator<Item = u64> + Send + 'a,
  ) -> Result<Vec<BytesPosition>> {
    let mut end_position: Option<u64> = None;
    let mut bytes_positions = Vec::new();

    for chunk in chunks {
      let maybe_end = positions.find(|pos| pos > &chunk.end().compressed());

      let end = match maybe_end {
        None => match end_position {
          None => {
            let pos = self.position_at_eof(id, format).await?;
            end_position = Some(pos);
            pos
          }
          Some(pos) => pos,
        },
        Some(pos) => pos,
      };

      bytes_positions.push(
        BytesPosition::default()
          .with_start(chunk.start().compressed())
          .with_end(end)
          .with_class(Body),
      )
    }

    Ok(bytes_positions)
  }

  /// Get unmapped bytes ranges.
  async fn get_byte_ranges_for_unmapped(
    &self,
    _id: &str,
    _format: &Format,
    _index: &Index,
  ) -> Result<Vec<BytesPosition>> {
    Ok(Vec::new())
  }
}

#[async_trait]
impl<S, ReaderType, ReferenceSequence, Index, Reader, Header, T>
  SearchAll<S, ReaderType, ReferenceSequence, Index, Reader, Header> for T
where
  S: Storage<Streamable = ReaderType> + Send + Sync + 'static,
  ReaderType: AsyncRead + Unpin + Send + Sync,
  Reader: Send + Sync,
  Header: FromStr + Send + Sync,
  <Header as FromStr>::Err: Display,
  ReferenceSequence: BinningIndexReferenceSequence + Sync,
  Index: BinningIndex + BinningIndexExt + Send + Sync,
  T: BgzfSearch<S, ReaderType, ReferenceSequence, Index, Reader, Header> + Send + Sync,
{
  async fn get_byte_ranges_for_all(
    &self,
    id: String,
    format: Format,
  ) -> Result<Vec<BytesPosition>> {
    Ok(vec![
      BytesPosition::default().with_end(self.position_at_eof(&id, &format).await?)
    ])
  }

  async fn get_header_end_offset(&self, index: &Index) -> Result<u64> {
    Self::index_positions(index)
      .into_iter()
      .next()
      .ok_or_else(|| {
        HtsGetError::io_error(format!(
          "finding header offset in `{}` index",
          self.get_format()
        ))
      })
  }

  fn get_eof_marker(&self) -> &[u8] {
    BGZF_EOF
  }

  fn get_eof_data_block(&self) -> Option<DataBlock> {
    Some(DataBlock::Data(Vec::from(BGZF_EOF), Some(Body)))
  }
}

/// Extension trait for binning indicies.
pub(crate) trait BinningIndexExt {
  /// Get all chunks associated with this index from the reference sequences.
  fn get_all_chunks(&self) -> Vec<&Chunk>;
}
