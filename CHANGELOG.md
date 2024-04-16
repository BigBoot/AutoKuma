# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- **!! WARNING POTENTIAL BREAKING CHANGE !!**: switch from custom templating to [Tera](https://keats.github.io/tera/docs/#introduction)  
  This change *might* break your current configuration if one of the following applies:
  - Your labels contain `{{`, `}}`, `{%`, `%}`, `{#` or `#}`:  
    Wrap your labels in `{% raw %}{% endraw %}` to ensure Tera does not try to parse them, e.g.:  
      `kuma.mygroup.group.name: "This is a {{ Group }}"` -> `kuma.mygroup.group.name: "{% raw %}This is a {{ Group }}{% endraw %}"`
  - You are using [Snippets](https://github.com/BigBoot/AutoKuma#snippets) with arguments:  
    The syntax for snippet arguments changed from `{{@0}}`, `{{@1}}`, `{{@2}}` etc. to `{{args[0]}}`, `{{args[1]}}`, `{{args[2]}}` etc.

### Added
- kuma-cli: docker-host management
- kuma-cli: colored output in pretty print mode

### Removed
- kuma-cli: toml output format as it's not compatible with all data

## [0.5.0] - 2024-04-13
### Fixed
- Use correct tag name when resolving groups, see [#17](https://github.com/BigBoot/AutoKuma/issues/17)
- Fix missing snake_case aliases, see [#19](https://github.com/BigBoot/AutoKuma/issues/19)

### Added
- lenient vector parsing for `accepted_statuscodes`, see [#23](https://github.com/BigBoot/AutoKuma/issues/23)
- support for `push_token`, see [#21](https://github.com/BigBoot/AutoKuma/issues/21)
- Snippets for reduced boilerplate

## [0.4.1] - 2024-04-02
### Fixed
- Release pipeline

## [0.4.0] - 2024-04-02
### Added
- Ability to replace template values in tag names (as long as the tags start with the defined prefix), see [#14](https://github.com/BigBoot/AutoKuma/issues/14)
- Ability to load user-wide AutoKuma config
- `log_dir` config for storing logs as files

### Fixed
- Fixed parsing of `max_retries` and `max_redirects`, see [#12](https://github.com/BigBoot/AutoKuma/issues/12)

### Changed
- Remove leading slash from container names in template values, see [#14](https://github.com/BigBoot/AutoKuma/issues/14)
- Added fallback for `static_monitors`

## [0.3.2] - 2024-03-28
### Added
- Fall back to `DOCKER_HOST` env variable when no socket_path is specified in AutoKuma docker config
- add docker image for kuma-cli, see [#5](https://github.com/BigBoot/AutoKuma/issues/5)

### Fixed
- exclude parent_name when sending monitor data to server, see [#8](https://github.com/BigBoot/AutoKuma/issues/8)
- Make parsing of ports more lenient, see [#9](https://github.com/BigBoot/AutoKuma/issues/9)

  
## [0.3.1] - 2024-02-27
### Fixed
- Memory leak in kuma-client [#1](https://github.com/BigBoot/AutoKuma/issues/1)
- Fix documentation for maintenance cli subcommand [#4](https://github.com/BigBoot/AutoKuma/issues/4)
- Allow deserialization of maintenance timerange without seconds [#4](https://github.com/BigBoot/AutoKuma/issues/4)

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

