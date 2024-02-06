# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Fixed
- Memory leak in kuma-client [#1](https://github.com/BigBoot/AutoKuma/issues/1)

## [0.3.0] - 2024-01-13
### Added
- new CLI client for Uptime Kuma `kuma-cli`

### Changed
- split package into `kuma-client` and `autokuma`
- renamed env var `AUTOKUMA__KUMA__TAG_NAME` to `AUTOKUMA__TAG_NAME` due to package splitting
- renamed env var `AUTOKUMA__KUMA__TAG_COLOR` to `AUTOKUMA__TAG_COLOR` due to package splitting
- renamed env var `AUTOKUMA__KUMA__DEFAULT_SETTINGS` to `AUTOKUMA__DEFAULT_SETTINGS` due to package splitting
- automatically append `/socket.io/` to `KUMA__URL`

## [0.2.0] - 2024-01-09
### Added
- Make Connection timeout configurable [`AUTOKUMA__KUMA__CONNECT_TIMEOUT`]
- Make Call timeout configurable [`AUTOKUMA__KUMA__CALL_TIMEOUT`]
- Allow defining default settings [`AUTOKUMA__KUMA__DEFAULT_SETTINGS`]
- Add templates for setting values
- Ability to load "static" monitors from a directory [`AUTOKUMA__STATIC_MONITORS`]
- Ability to disable docker intergration [`AUTOKUMA__DOCKER__ENABLED`] (useful in combination with `AUTOKUMA__STATIC_MONITORS`)


### Changed
- Increase default timeout for calls and connecting to 30s

### Fixed
- Support for authentication using credentials
- Fix typo in env variable: AUTOKUMA__DOCKER__LABEL_PREFOX -> AUTOKUMA__DOCKER__LABEL_PREFIX
- Websocket connections were not being closed correctly so they would just accumulate over time

## [0.1.1] - 2024-01-07

### Added
- Ability to parse nested structs from container labels (required for notification_id_list)

### Changed
- Improve error handling

## [0.1.0] - 2024-01-06

### Added
- First release

