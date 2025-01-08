# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.7.2...htsget-actix-v0.8.0) - 2025-01-08

### Added

- *(config)* add cargo package filled service info fields
- [**breaking**] add pre-filled package info, description and repository url to the service info endpoint

### Fixed

- service info group, artifact and version, and add flexibility in configuration

### Other

- rename s3-storage to aws and url-storage to url
- add location concept and move advanced config to its own module
- grammar and typos
- re-word and simplify, add quick starts where applicable

## [0.7.2](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.7.1...htsget-actix-v0.7.2) - 2024-10-22

### Other

- update Cargo.lock dependencies

## [0.7.1](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.7.0...htsget-actix-v0.7.1) - 2024-10-16

### Other

- update Cargo.lock dependencies

## [0.7.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.6.2...htsget-actix-v0.7.0) - 2024-09-30

### Added

- *(config)* add configuration values for C4GH S3 and Url storage
- Crypt4GH support using LocalStorage

### Fixed

- explicitly choose aws_lc_rs as the crypto provider

### Other

- rename type to backend and clarify experimental feature flag
- [**breaking**] rename c4gh-experimental to experimental
- [**breaking**] allow mutable search trait, use way less Arcs and clones
- Merge pull request [#259](https://github.com/umccr/htsget-rs/pull/259) from umccr/release-plz-2024-09-03T01-36-36Z
- [**breaking**] remove htsget-lambda library code and replace with axum router

## [0.6.2](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.6.1...htsget-actix-v0.6.2) - 2024-08-04

### Added
- *(axum)* add join handle helper functions

### Other
- update rust msrv
- *(actix)* clarify axum vs actix usage
- add routers for data and ticket servers
- move the data server to its own htsget-axum crate
- move storage module from htsget-search to htsget-storage

## [0.6.1](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.6.0...htsget-actix-v0.6.1) - 2024-05-22

### Other
- major dep updates, rustls 0.21 -> 0.23, http 0.2 -> 1, reqwest 0.11 -> 0.12, noodles 0.65 -> 0.74 + minor bumps for other crates depending on these.
- Merge branch 'main' of https://github.com/umccr/htsget-rs into mio_h2_bump

## [0.6.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.8...htsget-actix-v0.6.0) - 2024-05-19

### Other
- update MSRV
- [**breaking**] swap out hyper for reqwest to support redirects
- *(test)* remove server-tests and cors-tests features and create http module

## [0.5.8](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.7...htsget-actix-v0.5.8) - 2024-01-02

### Other
- *(deps)* update noodles to 0.60, new clippy warnings

## [0.5.7](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.6...htsget-actix-v0.5.7) - 2023-11-02

### Other
- updated the following local packages: htsget-config, htsget-search

## [0.5.6](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.5...htsget-actix-v0.5.6) - 2023-10-30

### Other
- updated the following local packages: htsget-search

## [0.5.5](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.4...htsget-actix-v0.5.5) - 2023-10-23

### Other
- update dependencies

## [0.5.4](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.3...htsget-actix-v0.5.4) - 2023-10-02

### Other
- update dependencies

## [0.5.3](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.2...htsget-actix-v0.5.3) - 2023-09-06

### Other
- revert htsget-test change to a dev dependency
- add pre-commit hook
- *(deps)* update msrv and attempt using htsget-test as a dev dependency
- bump deps including rustls 0.21

## [0.5.2](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.1...htsget-actix-v0.5.2) - 2023-09-05

### Other
- bump up deploy packages, also solves CVE-2023-35165 and CVE-2022-25883

## [0.5.1](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.5.0...htsget-actix-v0.5.1) - 2023-08-23

### Other
- update dependencies

## [0.5.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.5...htsget-actix-v0.5.0) - 2023-07-11

### Added
- [**breaking**] implement client tls config
- [**breaking**] add server config to certificate key pair

## [0.4.5](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.4...htsget-actix-v0.4.5) - 2023-06-25

### Other
- updated the following local packages: htsget-search

## [0.4.4](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.3...htsget-actix-v0.4.4) - 2023-06-20

### Other
- bump deps

## [0.4.3](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.2...htsget-actix-v0.4.3) - 2023-06-19

### Other
- updated the following local packages: htsget-config, htsget-search

## [0.4.2](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.1...htsget-actix-v0.4.2) - 2023-06-08

### Other
- remove unused dependencies and update msrv
- update remaining dependencies, hold back tokio-rustls due to conflicting versions with actix-web
- update noodles, remove repeated code in search

## [0.4.1](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.4.0...htsget-actix-v0.4.1) - 2023-06-02

### Fixed
- *(actix)* incorrect feature flags

### Other
- add debug line for config when starting server

## [0.3.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.2.0...htsget-actix-v0.3.0) - 2023-05-29

### Added
- format parsing is now case-insensitive when validating query parameters
- [**breaking**] add request header information to post handlers
- [**breaking**] add request header information to get handlers
- *(config)* add url-storage feature flag
- add option to format logs in different styles
- add error type to config
- *(actix)* TLS on ticket server
- [**breaking**] add tls config to ticket server, rearrange some fields
- *(test)* add multiple resolvers for server tests and test resolution
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Fixed
- inserting keys with the same name multiple times into headers serializes correctly
- *(config)* use set to avoid duplicate key-value pairs in headers
- [**breaking**] headers should allow multiple values for the same key
- use correct help context for a crate using htsget-config
- *(release)* Bump all crates to 0.1.2 as explored in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1422187766

### Other
- update for UrlStorage
- [**breaking**] rename AwsS3Storage to S3Storage in search
- [**breaking**] http refactor, pass request with query
- remove s3-storage as default
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/actix-tls
- a few style changes, changed default resolver
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config
- manually bump crate versions to 0.1.4
- make htsget-test a regular dependency
- bump crate versions to 0.1.3 manually
- specify htsget-test version
- *(test)* remove htsget-test dependence on htsget-search and htsget-http.
- [**breaking**] move CertificateKeyPair to config to simplify data server logic
- release
- Downgrade release after fixing cargo publish circular dep issues as discussed in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1420254400
- Update cargo files ([#152](https://github.com/umccr/htsget-rs/pull/152))
- *(deps)* bump tokio from 1.24.0 to 1.24.2 ([#151](https://github.com/umccr/htsget-rs/pull/151))
- release ([#148](https://github.com/umccr/htsget-rs/pull/148))
- Set MSRV on all sub-crates ([#146](https://github.com/umccr/htsget-rs/pull/146))
- Better CI ([#98](https://github.com/umccr/htsget-rs/pull/98))

## [0.2.0](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.1.4...htsget-actix-v0.2.0) - 2023-04-28

### Added
- *(test)* add multiple resolvers for server tests and test resolution
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Other
- a few style changes, changed default resolver
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config

## [0.1.1](https://github.com/umccr/htsget-rs/compare/htsget-actix-v0.1.0...htsget-actix-v0.1.1) - 2023-01-27

### Other
- Set MSRV on all sub-crates (#146)
- Better CI (#98)
