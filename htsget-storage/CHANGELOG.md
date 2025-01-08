# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2](https://github.com/umccr/htsget-rs/compare/htsget-storage-v0.2.1...htsget-storage-v0.2.2) - 2025-01-08

### Added

- *(config)* add cargo package filled service info fields

### Other

- rename s3-storage to aws and url-storage to url
- add location concept and move advanced config to its own module
- re-word and simplify, add quick starts where applicable

## [0.2.1](https://github.com/umccr/htsget-rs/compare/htsget-storage-v0.2.0...htsget-storage-v0.2.1) - 2024-10-16

### Added

- allow retrieving c4gh keys from secrets manager

### Fixed

- *(storage)* ensure c4gh data is read after determining the header size

### Other

- *(search)* remove unnecessary Arcs and unwraps in test code

## [0.2.0](https://github.com/umccr/htsget-rs/compare/htsget-storage-v0.1.1...htsget-storage-v0.2.0) - 2024-09-30

### Added

- *(storage)* allow S3 and Url storage to use C4GHStorage
- *(config)* add configuration values for C4GH S3 and Url storage
- Crypt4GH support using LocalStorage

### Fixed

- *(storage)* make overflow handling more robust
- build errors with conditional flags

### Other

- *(deps)* bump noodles and tower
- remove `ObjectType` in favour of a more flattened config hierarchy
- rename type to backend and clarify experimental feature flag
- *(config)* slight adjustment to reduce conditionally compiled code branches
- [**breaking**] make storage config more explicit
- *(config)* [**breaking**] remove object type and specify keys directly
- [**breaking**] rename c4gh-experimental to experimental
- [**breaking**] split out storage lib.rs into another types.rs module
- *(storage)* add c4gh storage tests
- *(storage)* use preprocess and postprocess for C4GH storage instead of Arc<Mutex<..>>
- [**breaking**] allow mutable search trait, use way less Arcs and clones

## [0.1.1](https://github.com/umccr/htsget-rs/compare/htsget-storage-v0.1.0...htsget-storage-v0.1.1) - 2024-08-05

### Other
- release

## [0.1.0](https://github.com/umccr/htsget-rs/releases/tag/htsget-storage-v0.1.0) - 2024-08-04

### Other
- update rust msrv
- add routers for data and ticket servers
- move the data server to its own htsget-axum crate
- move storage module from htsget-search to htsget-storage
