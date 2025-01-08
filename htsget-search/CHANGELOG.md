# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.2](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.9.1...htsget-search-v0.9.2) - 2025-01-08

### Added

- *(config)* add cargo package filled service info fields
- *(config)* implement path-based locations

### Other

- rename s3-storage to aws and url-storage to url
- add location concept and move advanced config to its own module
- grammar and typos
- re-word and simplify, add quick starts where applicable

## [0.9.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.9.0...htsget-search-v0.9.1) - 2024-10-16

### Added

- allow retrieving c4gh keys from secrets manager

### Other

- *(search)* remove unnecessary Arcs and unwraps in test code

## [0.9.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.8.1...htsget-search-v0.9.0) - 2024-09-30

### Added

- *(config)* add configuration values for C4GH S3 and Url storage
- Crypt4GH support using LocalStorage

### Other

- *(deps)* bump noodles and tower
- remove `ObjectType` in favour of a more flattened config hierarchy
- rename type to backend and clarify experimental feature flag
- [**breaking**] make storage config more explicit
- bump noodles and other dependencies
- [**breaking**] rename c4gh-experimental to experimental
- [**breaking**] split out storage lib.rs into another types.rs module
- *(storage)* use preprocess and postprocess for C4GH storage instead of Arc<Mutex<..>>
- [**breaking**] allow mutable search trait, use way less Arcs and clones

## [0.8.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.8.0...htsget-search-v0.8.1) - 2024-09-03

### Other
- release

## [0.8.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.7.1...htsget-search-v0.8.0) - 2024-08-04

### Other
- update rust msrv
- bump Lambda and noodles dependencies
- *(axum)* add server tests for axum ticket server
- add routers for data and ticket servers
- move storage module from htsget-search to htsget-storage

## [0.7.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.7.0...htsget-search-v0.7.1) - 2024-05-22

### Other
- major dep updates, rustls 0.21 -> 0.23, http 0.2 -> 1, reqwest 0.11 -> 0.12, noodles 0.65 -> 0.74 + minor bumps for other crates depending on these.
- Merge branch 'main' of https://github.com/umccr/htsget-rs into mio_h2_bump

## [0.7.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.6...htsget-search-v0.7.0) - 2024-05-19

### Added
- remove blacklisted_headers also from UrlStorage GET and HEAD requests.
- UrlStorage forwarded header blacklist

### Other
- cargo fmt
- update MSRV
- [**breaking**] swap out hyper for reqwest to support redirects
- *(refactor)* convert unwraps to Results inside concat module
- *(search)* additional chr11 and chr20 byte ranges tests for bam and cram
- all range tests now concatenate bytes, update CRAM files
- Merge branch 'main' of https://github.com/umccr/htsget-rs into test/concat-responses
- *(deps)* update noodles to 0.65

## [0.6.6](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.5...htsget-search-v0.6.6) - 2024-01-02

### Other
- *(deps)* further minor dependency changes
- *(deps)* update noodles to 0.60, new clippy warnings

## [0.6.5](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.4...htsget-search-v0.6.5) - 2023-11-02

### Added
- *(search)* remove response scheme because response url can specify it
- *(search)* add response url option to url storage

### Fixed
- *(s3)* properly handle first capture group bucket and add warnings for s3 errors

### Other
- Merge pull request [#216](https://github.com/umccr/htsget-rs/pull/216) from umccr/feat/response_url

## [0.6.4](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.3...htsget-search-v0.6.4) - 2023-10-30

### Other
- Cargo fmt, brrrr
- Fix clippy to newest 1.73 rust release and use secrets.GITHUB_TOKEN for the release workflow

## [0.6.3](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.2...htsget-search-v0.6.3) - 2023-10-02

### Fixed
- *(search)* use bcf test for bcf search
- *(search)* return empty response when reference name is in header but not in index instead of error

### Other
- Merge branch 'main' of https://github.com/umccr/htsget-rs into fix/error-no-index-pos
- *(search)* formatting

## [0.6.2](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.1...htsget-search-v0.6.2) - 2023-09-06

### Other
- *(deps)* fix htsget-test feature flags in htsget-search
- *(deps)* properly update noodles and aws dependencies
- revert htsget-test change to a dev dependency
- *(deps)* update msrv and attempt using htsget-test as a dev dependency
- bump deps including rustls 0.21
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/htsget-elsa

## [0.6.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.6.0...htsget-search-v0.6.1) - 2023-09-05

### Other
- updated the following local packages: htsget-test

## [0.6.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.5.3...htsget-search-v0.6.0) - 2023-07-11

### Added
- move hyper client construction to config and copy it to url storage
- [**breaking**] implement client tls config
- [**breaking**] add server config to certificate key pair
- [**breaking**] add stronger types for certificate key pairs
- introduce cert and key parsing into config

### Other
- Merge branch 'main' of https://github.com/umccr/htsget-rs into fix/http1

## [0.5.3](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.5.2...htsget-search-v0.5.3) - 2023-06-25

### Fixed
- *(search)* enable http/1.1 ALPN on UrlStorage client

## [0.5.2](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.5.1...htsget-search-v0.5.2) - 2023-06-20

### Other
- bump deps

## [0.5.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.5.0...htsget-search-v0.5.1) - 2023-06-19

### Fixed
- avoid overwriting forwarded headers in url when formatting response

### Other
- add tests for extending urls and headers
- *(search)* allow http or https url storage client

## [0.5.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.4.1...htsget-search-v0.5.0) - 2023-06-08

### Other
- nightly clippy suggestions
- remove unused dependencies and update msrv
- use http::Uri directly in config
- *(search)* swap out reqwest for hyper
- update remaining dependencies, hold back tokio-rustls due to conflicting versions with actix-web
- update noodles, remove repeated code in search

## [0.4.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.4.0...htsget-search-v0.4.1) - 2023-06-02

### Other
- updated the following local packages: htsget-config

## [0.3.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.2.0...htsget-search-v0.3.0) - 2023-05-29

### Added
- implement url formatting for url storage
- *(search)* [**breaking**] implement get request and streamable type for url storage
- *(search)* implement head request for url storage
- *(search)* get url from key
- *(search)* add url storage struct
- *(config)* use proper url parsing in config
- *(search)* include request headers to storage options
- *(search)* re-export some config types
- *(config)* add url-storage feature flag
- add error type to config
- *(search)* add pub rustls server config function
- *(test)* add multiple resolvers for server tests and test resolution
- use serve_at in data server rather than a constant
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Fixed
- inserting keys with the same name multiple times into headers serializes correctly
- local storage on windows no longer returns urls with incorrect paths
- *(search)* use url directly instead of converting to string first
- use correct help context for a crate using htsget-config
- *(search)* return error instead of panicking when a TLS key is not found
- *(release)* Bump all crates to 0.1.2 as explored in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1422187766
- fixes for byte range ends
- fix IOError name
- fixes & model improvements & bam search progress
- fixes & model improvements & bam search progress
- fix docs

### Other
- update for UrlStorage
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/url_storage
- *(search)* add authorization header check to test server
- *(search)* fix head content-length behaviour and add remaining storage tests
- *(search)* add mock server and request tests
- *(search)* add url format tests
- *(search)* add get url from key test
- [**breaking**] rename AwsS3Storage to S3Storage in search
- [**breaking**] http refactor, pass request with query
- remove s3-storage as default
- *(config)* rename ResolveResponse functions
- *(config)* add logic for url storage in resolvers
- Merge branch 'main' of https://github.com/umccr/htsget-rs into feat/actix-tls
- *(search)* return plain ServerConfig from rustls load function rather than Arc
- *(search)* convert match to if let
- *(search)* add warning when a non-valid PL read group header is found
- *(search)* add additional tests for searching resolvers and from storage
- *(search)* implement `ResolveResponse` on `HtsGetFromStorage`
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config
- manually bump crate versions to 0.1.4
- make htsget-test a regular dependency
- bump crate versions to 0.1.3 manually
- allow htsget-test to be published and bump deps
- specify htsget-test version
- [**breaking**] move CertificateKeyPair to config to simplify data server logic
- *(search)* apply rustfmt
- Merge pull request [#154](https://github.com/umccr/htsget-rs/pull/154) from umccr/release-plz/2023-02-07T21-45-42Z
- release
- Downgrade release after fixing cargo publish circular dep issues as discussed in https://github.com/MarcoIeni/release-plz/issues/507#issuecomment-1420254400
- Update cargo files ([#152](https://github.com/umccr/htsget-rs/pull/152))
- Remove s3-server and dependencies ([#150](https://github.com/umccr/htsget-rs/pull/150))
- *(deps)* bump tokio from 1.24.0 to 1.24.2 ([#151](https://github.com/umccr/htsget-rs/pull/151))
- *(fix)* Remove version from htsget-test and mark it for publish=false to avoid circular dependency as recommended by @Marcoleni in https://github.com/MarcoIeni/release-plz/pull/452#issuecomment-1409835221
- release ([#148](https://github.com/umccr/htsget-rs/pull/148))
- Set MSRV on all sub-crates ([#146](https://github.com/umccr/htsget-rs/pull/146))
- Better CI ([#98](https://github.com/umccr/htsget-rs/pull/98))
- Merge branch 'main' of https://github.com/umccr/htsget-rs into more-flexible-config-rename
- add test for long resolvers from environment variable config
- clippy and fmt
- allow specifying tags, reference names, fields with an 'All' value
- reduce some options for cors, remove repeated code when configuring cors
- fix tests affected by config, change some default values and move around config options
- apply changes to other crates from reworked config
- deserialize empty string as None value
- move cors config to separate file
- fix errors relating to new config
- add tests for checking for contained value in interval
- add safe cast for conversion between i64 and u64
- move fields, tags, no tags, query, and interval to config
- Make search structs public
- Remove ReferenceSequenceInfo
- Remove unused code and logic
- Add tests for no end position
- Remove requirement for default end position when converting to noodles interval
- Merge branch 'main' of https://github.com/umccr/htsget-rs into exports
- Export some functions for use as a dependency
- Re-export htsget-config as a dependency from other crates
- Update non-noodles dependencies
- Http and tls server test uses test utils
- Convert preflight server test to test utils
- Convert data server test to use test utils
- Implement http test traits for data server
- Add CorsLayer responses to htsget-http-lambda
- Move configure_cors to module file
- Bump deps for noodles and simplify logic around maximum reference sequence length using new noodles types
- List out allowed methods rather than sending wildcard
- Add ticket server cors tests
- Add cors options request test
- Fix tests implementation
- Can't use base directory function in test
- Implement additional cors test
- Add base path only function in tests for code reuse
- Add cors tests
- Layer should go after merge
- Implement cors for data server
- Rename ticket server to data server
- Move data server config to separate struct
- Implement cors for htsget-http-actix.
- Changes to deployment ([#116](https://github.com/umccr/htsget-rs/pull/116))
- Remove some trace log details to avoid making them overly long.
- Add more detail to gzi traces.
- Add more spans and tracing calls.
- Add buffered reading to bai and gzi.
- Add buffered reading to cram search.
- Remove logging chunks as it is too noisy.
- Emit trace logs from functions.
- Add some more instrument targets, use span in_scope.
- Remove sleep call.
- Add a few more tracing span targets.
- Add span tracing to test timings.
- Small changes related to indices ([#114](https://github.com/umccr/htsget-rs/pull/114))
- Bump dependencies, fix clippy warnings.
- Avoid reading index unless it is required.
- Remove some unnecessary unwraps.
- Improve errors so that they are more informative.
- Remove RangeBounds on BytesPosition as its use is less readable with classes.
- Fix tests related to response class.
- Remove headers from response if empty.
- Simplify JsonUrl logic.
- Responses should contain a class for all ranges, or no ranges.
- Fix tests.
- Implement RangeBounds for BytesPosition.
- Perform byte position merging when creating data blocks.
- Byte position records class, Header for only header bytes, Body for only body bytes, and None if there is a mix of bytes.
- Allow BytesPosition to record its own class.
- Fix unneseccary storage queries ([#107](https://github.com/umccr/htsget-rs/pull/107))
- Simplify querying for all records by determining file size.
- Server benchmarks should use non-tls ticket server as this is a fairer comparison to the htsget-refserver.
- Clean up code, format, update dependencies.
- Tests run independently by using dynamic port allocation.
- The GC info field should have a Float type rather than an Integer type.
- Implement non-tls ticket server alongside tls ticket server.
- Rename some traits and structs to clarify their purpose.
- Bump many deps (except querymap) and avoid pulling full tokio in, we just need macros and rt-multi-thread ([#96](https://github.com/umccr/htsget-rs/pull/96))
- Out of order urls ([#95](https://github.com/umccr/htsget-rs/pull/95))
- Pinning to noodles-tabix =0.9.0 as suggested in https://github.com/zaeleus/noodles/issues/90#issuecomment-1150361623 as a result of getting CI errors on https://github.com/umccr/htsget-rs/runs/6803593182?check_suite_focus=true#step:6:90
- Fix eof errors ([#87](https://github.com/umccr/htsget-rs/pull/87))
- Add benchmarks ([#59](https://github.com/umccr/htsget-rs/pull/59))
- Fix localstorage path ([#86](https://github.com/umccr/htsget-rs/pull/86))
- Fix tests and errors ([#83](https://github.com/umccr/htsget-rs/pull/83))
- Deploy htsget-http-lambda. ([#81](https://github.com/umccr/htsget-rs/pull/81))
- Enable choosing between storage types. ([#80](https://github.com/umccr/htsget-rs/pull/80))
- Remove file from localstorage ([#79](https://github.com/umccr/htsget-rs/pull/79))
- Spawn s3-server once so that tests don't have to be run on one thread. ([#78](https://github.com/umccr/htsget-rs/pull/78))
- Remove blocking ([#77](https://github.com/umccr/htsget-rs/pull/77))
- Htsget http lambda ([#76](https://github.com/umccr/htsget-rs/pull/76))
- Storage class for s3 ([#74](https://github.com/umccr/htsget-rs/pull/74))
- Decouple File struct from Search trait. ([#70](https://github.com/umccr/htsget-rs/pull/70))
- Fix runtime panics from curl ([#69](https://github.com/umccr/htsget-rs/pull/69))
- Bump all tokio versions and stay on track with Noodles versioning instead of working from its git /cc @andrewpatto
- Implement id resolver ([#60](https://github.com/umccr/htsget-rs/pull/60))
- Convert Storage and HtsGet traits to use async/await ([#56](https://github.com/umccr/htsget-rs/pull/56))
- Bump up noodles across crates, otherwise several versions get mixed up
- Add the service info endpoints ([#54](https://github.com/umccr/htsget-rs/pull/54))
- Add the htsget-http-core and htsget-http-actix crates ([#45](https://github.com/umccr/htsget-rs/pull/45))
- Track crates.io version of noodles ([#53](https://github.com/umccr/htsget-rs/pull/53))
- Use file size for end bytes ranges.
- Refactor commonalities across all formats.
- Providing the file size through the Storage abstraction. ([#49](https://github.com/umccr/htsget-rs/pull/49))
- Implement CRAM search backend. ([#44](https://github.com/umccr/htsget-rs/pull/44))
- Add BCF support ([#43](https://github.com/umccr/htsget-rs/pull/43))
- Improve the bytes ranges for the BAM header ([#42](https://github.com/umccr/htsget-rs/pull/42))
- VCF search interface implementation (/variants endpoint) ([#37](https://github.com/umccr/htsget-rs/pull/37))
- Adapt tests to noodles changes ([#41](https://github.com/umccr/htsget-rs/pull/41))
- Htsget tests ([#40](https://github.com/umccr/htsget-rs/pull/40))
- Calculate BAM byte ranges more accurately ([#35](https://github.com/umccr/htsget-rs/pull/35))
- Remove duplicity in Query, UrlOptions and Url ([#29](https://github.com/umccr/htsget-rs/pull/29))
- Fix Local Storage always adding the Range header ([#31](https://github.com/umccr/htsget-rs/pull/31))
- Storage model tests ([#28](https://github.com/umccr/htsget-rs/pull/28))
- Merge pull request [#25](https://github.com/umccr/htsget-rs/pull/25) from chris-zen/vcf_bcf_test_data
- Add some tests for LocalStorage::url ([#22](https://github.com/umccr/htsget-rs/pull/22))
- Add the rest of the builder methods to Query ([#21](https://github.com/umccr/htsget-rs/pull/21))
- Implement class attribute for reads. ([#19](https://github.com/umccr/htsget-rs/pull/19))
- Update references and README ([#18](https://github.com/umccr/htsget-rs/pull/18))
- add tests for BytesRange ([#14](https://github.com/umccr/htsget-rs/pull/14))
- merge byte ranges
- some renames
- Fix BAM search for unmapped & clippy errors
- add search by reference name and range
- add TODO for BamSearch::url tests
- add TODO for BamSearch::url tests
- added test for HtsGetFromStorage
- added 2 tests for BamSearch
- preparing tests for BamSearch
- add tests and fixes for LocalStorage
- reorganized + rustdocs
- introduced the concept of Storage
- failed attempt to use a BAI index
- work in progress

## [0.2.0](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.1.4...htsget-search-v0.2.0) - 2023-04-28

### Added
- *(test)* add multiple resolvers for server tests and test resolution
- use serve_at in data server rather than a constant
- *(config)* add automatic config inference for local and s3 storage, and rearrange modules
- [**breaking**] simplify storage config by allowing untagged enum representation

### Other
- *(search)* convert match to if let
- *(search)* add warning when a non-valid PL read group header is found
- *(search)* add additional tests for searching resolvers and from storage
- *(search)* implement `ResolveResponse` on `HtsGetFromStorage`
- [**breaking**] rename `HttpTicketFormatter` and remove `UrlFormatter` implementation for it
- [**breaking**] move htsget structs to config, and resolve storage type in config

## [0.1.1](https://github.com/umccr/htsget-rs/compare/htsget-search-v0.1.0...htsget-search-v0.1.1) - 2023-01-27

### Fixed
- fixes for byte range ends
- fix IOError name
- fixes & model improvements & bam search progress
- fixes & model improvements & bam search progress
- fix docs

### Other
- Set MSRV on all sub-crates (#146)
- Better CI (#98)
- Merge branch 'main' of https://github.com/umccr/htsget-rs into more-flexible-config-rename
- add test for long resolvers from environment variable config
- clippy and fmt
- allow specifying tags, reference names, fields with an 'All' value
- reduce some options for cors, remove repeated code when configuring cors
- fix tests affected by config, change some default values and move around config options
- apply changes to other crates from reworked config
- deserialize empty string as None value
- move cors config to separate file
- fix errors relating to new config
- add tests for checking for contained value in interval
- add safe cast for conversion between i64 and u64
- move fields, tags, no tags, query, and interval to config
- Make search structs public
- Remove ReferenceSequenceInfo
- Remove unused code and logic
- Add tests for no end position
- Remove requirement for default end position when converting to noodles interval
- Merge branch 'main' of https://github.com/umccr/htsget-rs into exports
- Export some functions for use as a dependency
- Re-export htsget-config as a dependency from other crates
- Update non-noodles dependencies
- Http and tls server test uses test utils
- Convert preflight server test to test utils
- Convert data server test to use test utils
- Implement http test traits for data server
- Add CorsLayer responses to htsget-http-lambda
- Move configure_cors to module file
- Bump deps for noodles and simplify logic around maximum reference sequence length using new noodles types
- List out allowed methods rather than sending wildcard
- Add ticket server cors tests
- Add cors options request test
- Fix tests implementation
- Can't use base directory function in test
- Implement additional cors test
- Add base path only function in tests for code reuse
- Add cors tests
- Layer should go after merge
- Implement cors for data server
- Rename ticket server to data server
- Move data server config to separate struct
- Implement cors for htsget-http-actix.
- Changes to deployment (#116)
- Remove some trace log details to avoid making them overly long.
- Add more detail to gzi traces.
- Add more spans and tracing calls.
- Add buffered reading to bai and gzi.
- Add buffered reading to cram search.
- Remove logging chunks as it is too noisy.
- Emit trace logs from functions.
- Add some more instrument targets, use span in_scope.
- Remove sleep call.
- Add a few more tracing span targets.
- Add span tracing to test timings.
- Small changes related to indices (#114)
- Bump dependencies, fix clippy warnings.
- Avoid reading index unless it is required.
- Remove some unnecessary unwraps.
- Improve errors so that they are more informative.
- Remove RangeBounds on BytesPosition as its use is less readable with classes.
- Fix tests related to response class.
- Remove headers from response if empty.
- Simplify JsonUrl logic.
- Responses should contain a class for all ranges, or no ranges.
- Fix tests.
- Implement RangeBounds for BytesPosition.
- Perform byte position merging when creating data blocks.
- Byte position records class, Header for only header bytes, Body for only body bytes, and None if there is a mix of bytes.
- Allow BytesPosition to record its own class.
- Fix unneseccary storage queries (#107)
- Simplify querying for all records by determining file size.
- Server benchmarks should use non-tls ticket server as this is a fairer comparison to the htsget-refserver.
- Clean up code, format, update dependencies.
- Tests run independently by using dynamic port allocation.
- The GC info field should have a Float type rather than an Integer type.
- Implement non-tls ticket server alongside tls ticket server.
- Rename some traits and structs to clarify their purpose.
- Bump many deps (except querymap) and avoid pulling full tokio in, we just need macros and rt-multi-thread (#96)
- Out of order urls (#95)
- Pinning to noodles-tabix =0.9.0 as suggested in https://github.com/zaeleus/noodles/issues/90#issuecomment-1150361623 as a result of getting CI errors on https://github.com/umccr/htsget-rs/runs/6803593182?check_suite_focus=true#step:6:90
- Fix eof errors (#87)
- Add benchmarks (#59)
- Fix localstorage path (#86)
- Fix tests and errors (#83)
- Deploy htsget-http-lambda. (#81)
- Enable choosing between storage types. (#80)
- Remove file from localstorage (#79)
- Spawn s3-server once so that tests don't have to be run on one thread. (#78)
- Remove blocking (#77)
- Htsget http lambda (#76)
- Storage class for s3 (#74)
- Decouple File struct from Search trait. (#70)
- Fix runtime panics from curl (#69)
- Bump all tokio versions and stay on track with Noodles versioning instead of working from its git /cc @andrewpatto
- Implement id resolver (#60)
- Convert Storage and HtsGet traits to use async/await (#56)
- Bump up noodles across crates, otherwise several versions get mixed up
- Add the service info endpoints (#54)
- Add the htsget-http-core and htsget-http-actix crates (#45)
- Track crates.io version of noodles (#53)
- Use file size for end bytes ranges.
- Refactor commonalities across all formats.
- Providing the file size through the Storage abstraction. (#49)
- Implement CRAM search backend. (#44)
- Add BCF support (#43)
- Improve the bytes ranges for the BAM header (#42)
- VCF search interface implementation (/variants endpoint) (#37)
- Adapt tests to noodles changes (#41)
- Htsget tests (#40)
- Calculate BAM byte ranges more accurately (#35)
- Remove duplicity in Query, UrlOptions and Url (#29)
- Fix Local Storage always adding the Range header (#31)
- Storage model tests (#28)
- Merge pull request #25 from chris-zen/vcf_bcf_test_data
- Add some tests for LocalStorage::url (#22)
- Add the rest of the builder methods to Query (#21)
- Implement class attribute for reads. (#19)
- Update references and README (#18)
- add tests for BytesRange (#14)
- merge byte ranges
- some renames
- Fix BAM search for unmapped & clippy errors
- add search by reference name and range
- add TODO for BamSearch::url tests
- add TODO for BamSearch::url tests
- added test for HtsGetFromStorage
- added 2 tests for BamSearch
- preparing tests for BamSearch
- add tests and fixes for LocalStorage
- reorganized + rustdocs
- introduced the concept of Storage
- failed attempt to use a BAI index
- work in progress
