# Architecture

This document describes the high-level architecture of htsget-rs. If you want to familiarize yourself with the code base, you are in the right place!

See also the official [htsget paper][htsget-paper] and [htsget's specification][htsget-spec] which describe how requests and responses should look like.

## Birds eye view

This repository implements the [htsget specification][htsget-spec] as closely as possible with Rust. The data exchange resembles the diagram below, outlined in the [official spec][htsget-spec]:

![htsget-ticket][htsget-ticket]

On the highest level, htsget-rs receives queries about genes or other bioinformatic features from a client and the server returns data from various bioinformatics data formats saved on a storage backend. In other words, htsget is the abstraction layer sitting between your client/visualizer/pipeline and the gory details of 10+ year old bioinformatics data formats.

## Code Map

This repository consists of a workspace composed by the following crates:

- [htsget-search](htsget-search): Core logic needed to run searches in the genomic data according to the htsget specs: genomic data via reads variants from cloud storage, run queries on their indices. Other interfaces can be build outside of this crate but on top of this core functionality. 
- [htsget-http-core](htsget-http-core): Handling of htsget's HTTP requests: converting query results to JSON, client error reporting. Aims contain everything HTTP related that isn't framework dependent.
- [htsget-http-actix](htsget-http-actix): This crate contains a working server implementation based on the other crates in the project. It contains the framework dependent code. It should be possible for anyone to write another crate like this one using htsget-search, htsget-http-core and their preferred framework;
- [htsget-devtools](htsget-devtools): This is just a bunch of code helping us to explore the formats or to proof some concepts.

More crates will come as we progress in this project, such as the htsget id resolver interface layer.

### htsget-search

This crate provides two basic abstractions:

- [htsget](htsget-search/src/htsget/mod.rs#L18): The `htsget` trait represents an entity that can resolve queries according to the htsget spec.
  The `htsget` trait comes together with a basic model to represent basic entities needed to perform a search (`Query`, `Format`, `Class`, `Tags`, `Headers`, `Url`, `Response`).
  We include a reference implementation called [HtsGetFromStorage](htsget-search/src/htsget/from_storage.rs) that provides the logic to resolve queries using an external `Storage`.
  It supports the [BAM](htsget-search/src/htsget/bam_search.rs), [BCF](htsget-search/src/htsget/bcf_search.rs), [CRAM](htsget-search/src/htsget/cram_search.rs), and [VCF](htsget-search/src/htsget/vcf_search.rs) formats.

- [storage](htsget-search/src/storage/mod.rs): The `Storage` trait represents some kind of object based storage (either locally or in the cloud) that can be used to retrieve files for alignments, variants or its respective indexes, as well as to get metadata from them. We include a reference implementation using [local files](htsget-search/src/storage/local.rs), and [AWS S3](https://github.com/chris-zen/htsget-mvp/issues/9).

#### Traits abstraction

The htsget `Search` trait is an abstraction over all the formats, which removes commonalities. While each format has 
its own particularities, there are many shared components that can be abstracted. Specifically, the `Search` trait defines
the interface that handles the core logic of a htsget search request, and it passes the format specifics to the individual 
format implementations.

The implemention depends heavily on the [noodles bioinformatics crate](https://github.com/zaeleus/noodles), which handles the underlying data processing.
Any changes to file format specifications would likely be reflected in the noodles crate, and minimal changes would be required in [htsget-search](htsget-search), due 
to the interface that noodles provides.

# References

For a deep dive on the aforementioned bioinformatics data formats, here are links to the official specifications:

### SAM/BAM formats and tools

[SAM specification](https://github.com/samtools/hts-specs/blob/master/SAMv1.pdf)
[The great *noodles* library](https://github.com/zaeleus/noodles)
[Inspecting, summarizing, and manipulating the read alignments](https://mtbgenomicsworkshop.readthedocs.io/en/latest/material/day3/mappingstats.html)

### VCF/BCF formats

[VCF specification](https://samtools.github.io/hts-specs/VCFv4.3.pdf)

## Previous attempts to work on htsget with Rust

- https://github.com/brainstorm/htsget-indexer
- https://github.com/brainstorm/bio-index-formats/


[htsget-spec]: https://samtools.github.io/hts-specs/htsget.html
[htsget-ticket]: https://samtools.github.io/hts-specs/pub/htsget-ticket.png
[htsget-paper]: https://academic.oup.com/bioinformatics/article/35/1/119/5040320

## Improving this document further

Adding the gaps explained in the [ARCHITECTURE blogpost](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html):

* WHERE to change the code, given feature X, give pointers.
* Do name important files, modules, and types: NO LINKS (links go stale), JUST NAMES (symbol search).
* Call-out architectural invariants: explain absence of something.
* Point out boundaries between layers and systems.