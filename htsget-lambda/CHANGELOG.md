# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.5.2...htsget-lambda-v0.6.0) - 2025-01-08

### Added

- *(config)* add cargo package filled service info fields
- [**breaking**] add pre-filled package info, description and repository url to the service info endpoint

### Other

- rename s3-storage to aws and url-storage to url
- add location concept and move advanced config to its own module
- re-word and simplify, add quick starts where applicable

## [0.5.2](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.5.1...htsget-lambda-v0.5.2) - 2024-10-22

### Other

- update Cargo.lock dependencies

## [0.5.1](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.5.0...htsget-lambda-v0.5.1) - 2024-10-16

### Other

- update Cargo.lock dependencies

## [0.5.0](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.17...htsget-lambda-v0.5.0) - 2024-09-30

### Added

- *(config)* add configuration values for C4GH S3 and Url storage
- Crypt4GH support using LocalStorage

### Fixed

- explicitly choose aws_lc_rs as the crypto provider

### Other

- *(deps)* bump noodles and tower
- rename type to backend and clarify experimental feature flag
- [**breaking**] rename c4gh-experimental to experimental
- Merge pull request [#259](https://github.com/umccr/htsget-rs/pull/259) from umccr/release-plz-2024-09-03T01-36-36Z
- add more server tests and fix Dockerfile.dockerignore
- [**breaking**] remove htsget-lambda library code and replace with axum router

## [0.4.17](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.16...htsget-lambda-v0.4.17) - 2024-08-04

### Other
- update rust msrv
- bump Lambda and noodles dependencies
- add routers for data and ticket servers
- move the data server to its own htsget-axum crate
- move storage module from htsget-search to htsget-storage

## [0.4.16](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.15...htsget-lambda-v0.4.16) - 2024-05-22

### Other
- major dep updates, rustls 0.21 -> 0.23, http 0.2 -> 1, reqwest 0.11 -> 0.12, noodles 0.65 -> 0.74 + minor bumps for other crates depending on these.
- Merge branch 'main' of https://github.com/umccr/htsget-rs into mio_h2_bump

## [0.4.15](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.14...htsget-lambda-v0.4.15) - 2024-05-19

### Other
- update MSRV
- *(test)* remove server-tests and cors-tests features and create http module

## [0.4.14](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.13...htsget-lambda-v0.4.14) - 2024-01-02

### Other
- *(deps)* update noodles to 0.60, new clippy warnings

## [0.4.13](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.12...htsget-lambda-v0.4.13) - 2023-11-02

### Other
- updated the following local packages: htsget-config, htsget-search

## [0.4.12](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.11...htsget-lambda-v0.4.12) - 2023-10-30

### Other
- updated the following local packages: htsget-search

## [0.4.11](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.10...htsget-lambda-v0.4.11) - 2023-10-23

### Other
- update dependencies

## [0.4.10](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.9...htsget-lambda-v0.4.10) - 2023-10-02

### Other
- update dependencies

## [0.4.9](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.8...htsget-lambda-v0.4.9) - 2023-09-06

### Other
- revert htsget-test change to a dev dependency
- *(deps)* update msrv and attempt using htsget-test as a dev dependency
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/htsget-elsa

## [0.4.8](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.7...htsget-lambda-v0.4.8) - 2023-09-05

### Other
- update dependencies

## [0.4.7](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.6...htsget-lambda-v0.4.7) - 2023-08-23

### Other
- update dependencies

## [0.4.6](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.5...htsget-lambda-v0.4.6) - 2023-07-11

### Other
- update dependencies

## [0.4.5](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.4...htsget-lambda-v0.4.5) - 2023-06-25

### Other
- updated the following local packages: htsget-search

## [0.4.4](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.3...htsget-lambda-v0.4.4) - 2023-06-20

### Other
- bump deps

## [0.4.3](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.2...htsget-lambda-v0.4.3) - 2023-06-19

### Other
- updated the following local packages: htsget-config, htsget-search

## [0.4.2](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.1...htsget-lambda-v0.4.2) - 2023-06-08

### Other
- remove unused dependencies and update msrv
- update remaining dependencies, hold back tokio-rustls due to conflicting versions with actix-web

## [0.4.1](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.4.0...htsget-lambda-v0.4.1) - 2023-06-02

### Other
- add debug line for config when starting server

## [0.3.0](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.2.0...htsget-lambda-v0.3.0) - 2023-05-29

### Added
- format parsing is now case-insensitive when validating query parameters
- [**breaking**] add request header information to post handlers
- [**breaking**] add request header information to get handlers
- *(config)* add url-storage feature flag
- add option to format logs in different styles
- add error type to config
- [**breaking**] add tls config to ticket server, rearrange some fields
- *(test)* add multiple resolvers for server tests and test resolution
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Fixed
- use correct help context for a crate using htsget-config
- *(release)* Bump all crates to 0.1.2 as explored in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1422187766

### Other
- update for UrlStorage
- [**breaking**] rename AwsS3Storage to S3Storage in search
- [**breaking**] http refactor, pass request with query
- remove s3-storage as default
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/actix-tls
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config
- manually bump crate versions to 0.1.4
- make htsget-test a regular dependency
- bump crate versions to 0.1.3 manually
- specify htsget-test version
- *(test)* remove htsget-test dependence on htsget-search and htsget-http.
- [**breaking**] move CertificateKeyPair to config to simplify data server logic
- Merge pull request [#133](https://github.com/umccr/htsget-rs/pull/133) from umccr/deploy-htsget-rs
- release
- Downgrade release after fixing cargo publish circular dep issues as discussed in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1420254400
- Update cargo files ([#152](https://github.com/umccr/htsget-rs/pull/152))
- Remove s3-server and dependencies ([#150](https://github.com/umccr/htsget-rs/pull/150))
- *(deps)* bump tokio from 1.24.0 to 1.24.2 ([#151](https://github.com/umccr/htsget-rs/pull/151))
- release ([#148](https://github.com/umccr/htsget-rs/pull/148))
- Set MSRV on all sub-crates ([#146](https://github.com/umccr/htsget-rs/pull/146))
- Better CI ([#98](https://github.com/umccr/htsget-rs/pull/98))

## [0.2.0](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.1.4...htsget-lambda-v0.2.0) - 2023-04-28

### Added
- *(test)* add multiple resolvers for server tests and test resolution
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Other
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config

## [0.1.1](https://github.com/umccr/htsget-rs/compare/htsget-lambda-v0.1.0...htsget-lambda-v0.1.1) - 2023-01-27

### Other
- Set MSRV on all sub-crates (#146)
- Better CI (#98)
