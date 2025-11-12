# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2025-11-12
### Changed
- ⚠️ BREAKING CHANGE ⚠️ Uptime Kuma v2 support is now enabled by default, v1 compatible images are available with the `uptime-kuma-v1-` prefix, please make sure to use the correct tag.
- AutoKuma now keeps the connection alive between syncs, this should dramatically reduce the load on uptime kuma
- Improve error messages for label parsing

### Fixed
- Improve parsing for `expected_value` properties
- DateTime parsing now works for MariaDB backed uptime kuma instances
- Improve parsing for `notification.active` and `notifcation.is_default`
- Default values not being applied correctly

### Added
- Add container exclusion patterns for Docker Compose temporary containers, see #141 [@Laptop765](https://github.com/Laptop765)
- Remote Database Shrink and Remote Database GetSize command, see #144 [@ianmclinden](https://github.com/ianmclinden)

## [1.0.0] - 2025-05-09
### Changed
- ⚠️ BREAKING CHANGE ⚠️ AutoKuma now requires persistent storage:  
  AutoKuma previously relied on using an Uptime Kuma label on monitors to store internal data, allowing it to operate statelessly without requiring storage.  
  However, this approach had significant drawbacks: it always added a label to monitors managed by AutoKuma and only worked for monitors, not for other entities like notification providers, docker hosts, labels, etc.  
  Starting with this version, AutoKuma will instead track its monitors using a database that needs to be stored in a persistent location.  
  For a Docker Compose setup, you can use a named volume. Please refer to the [example docker-compose.yml](https://github.com/BigBoot/AutoKuma#example-docker-compose-) for guidance.  
  
  To prevent data loss, AutoKuma will refuse to run if it detects any old monitors with an AutoKuma label. After adding a persistent storage location, AutoKuma will need to be started once with the environment variable `AUTOKUMA__MIGRATE=true` set to take over any existing monitors. This environment variable can be removed afterward.
- Access to environment variables in templates restricted to variables starting with `AUTOKUMA__ENV__`, see [#97](https://github.com/BigBoot/AutoKuma/issues/97)
- Changed Monitor::proxy_id to Option<i32>, parsing now supports both numbers and strings, see [#95](https://github.com/BigBoot/AutoKuma/issues/95)
- File source now skips hidden files 
- File source now skips unsupported file types, see [#127](https://github.com/BigBoot/AutoKuma/issues/127)

### Added
- All CLI commands now support reading multiple files/ids/slugs at once, as well as parsing arrays of objects where applicable, [#74](https://github.com/BigBoot/AutoKuma/issues/74)
- Access to `system_info` in templates, see [#2]
- Ability to generate TOTP tokens from a TOTP secret
- Support for creating Tags from Labels, see [#40](https://github.com/BigBoot/AutoKuma/issues/40)
- File source now supports multiple entities per file, the ids will be named `<file_stem>[<index>]` accordingly, see [#86](https://github.com/BigBoot/AutoKuma/issues/86)
- Kubernetes integration, see [#58](https://github.com/BigBoot/AutoKuma/issues/58)
- File source can now follow symlinks (configurable, disabled by default)
- Add `--mfa-secret` to kuma CLI
- Add authentication using a jwt token:
  - AutoKuma will store a jwt token after the initial login and use this for subsequent logins, alternatively a token can be supplied manually in the config
  - Kuma CLI now has a `login` command which allows manually obtaining a jwt token
  - Kuma CLI now has a `--auth_token` option to allow authenticating with a jwt token
  - Kuma CLI now has a `--store-auth` option which will store the token after a succesfull login (e.g. using `kuma login --store-auth [username]`)
- Added grace period before removing entities when the definitions vanish (e.g. when updating a container)

### Fixed
- File source now skips unsupported files rather then interrupting the whole sync, see [#89](https://github.com/BigBoot/AutoKuma/issues/89)
- File source now supports nested folders, the ids will be in the format `<folder>/<filename>` without the file extension, see [#28](https://github.com/BigBoot/AutoKuma/issues/28)
- Add missing retry_interval alias, see [#96](https://github.com/BigBoot/AutoKuma/issues/96)
- Fix wrong path for websocket url, see [#123](https://github.com/BigBoot/AutoKuma/issues/123)

## [0.8.0] - 2024-08-22
### Added
- Support for multiple docker hosts, see [#62](https://github.com/BigBoot/AutoKuma/issues/62)
- Ability to specify custom tls cert when connecting to Uptime Kuma over HTTPS, see [#66](https://github.com/BigBoot/AutoKuma/issues/66)
- Ability to ignore invalid tls certs when connecting to Uptime Kuma over HTTPS, see [#66](https://github.com/BigBoot/AutoKuma/issues/66)

### Fixed
- Fix control sequences not being expanded in snippets, see [#60](https://github.com/BigBoot/AutoKuma/issues/60)
- Add missing `description` property, see [#61](https://github.com/BigBoot/AutoKuma/issues/61)
- Fix wrong name for `query` property of database monitors, see [#65](https://github.com/BigBoot/AutoKuma/issues/65)
- Add missing `upside_down` property alias, see [#71](https://github.com/BigBoot/AutoKuma/issues/71)
- Fix possible memory leak, see [#41](https://github.com/BigBoot/AutoKuma/issues/41)
- Add workaround for ntfy, see [#72](https://github.com/BigBoot/AutoKuma/issues/72)

## [0.7.0] - 2024-07-15
### Added
- Enable SSL for docker connections (using DOCKER_HOST, DOCKER_CERT_PATH, DOCKER_TLS_VERIFY env vars), see [#50](https://github.com/BigBoot/AutoKuma/issues/50)
- Create monitors from docker swarm services, see [#54](https://github.com/BigBoot/AutoKuma/issues/54)
- `create_paused` property to create new monitors in paused state, see [#53](https://github.com/BigBoot/AutoKuma/issues/53)

### Fixed
- Add missing `query` property for `sqlserver`, `postgres` and `mysql`, see [#36](https://github.com/BigBoot/AutoKuma/issues/36)
- Fix parsing of recurring maintenance with no effective date range, see [#43](https://github.com/BigBoot/AutoKuma/issues/43)
- Filter cyclic parent_name reference, see [#56](https://github.com/BigBoot/AutoKuma/issues/56)

### Changed
- Transparently flatten arrays in static monitor definitions, see [#51](https://github.com/BigBoot/AutoKuma/issues/51)

## [0.6.0] - 2024-04-28
### Changed
- **!! WARNING POTENTIAL BREAKING CHANGE !!**: switch from custom templating to [Tera](https://keats.github.io/tera/docs/#introduction)  
  This change *might* break your current configuration if one of the following applies:
  - Your labels contain `{{`, `}}`, `{%`, `%}`, `{#` or `#}`:  
    Wrap your labels in `{% raw %}{% endraw %}` to ensure Tera does not try to parse them, e.g.:  
      `kuma.mygroup.group.name: "This is a {{ Group }}"` -> `kuma.mygroup.group.name: "{% raw %}This is a {{ Group }}{% endraw %}"`
  - You are using [Snippets](https://github.com/BigBoot/AutoKuma#snippets) with arguments:  
    The syntax for snippet arguments changed from `{{@0}}`, `{{@1}}`, `{{@2}}` etc. to `{{args[0]}}`, `{{args[1]}}`, `{{args[2]}}` etc.
- Static Monitors will be be postprocessed in the same way as monitors defined by container labels, see [#33](https://github.com/BigBoot/AutoKuma/issues/33)

### Fixed
- autokuma: connections not being closed when an error occurs during setup, see [#11](https://github.com/BigBoot/AutoKuma/issues/11)

### Added
- kuma-cli: docker-host management
- kuma-cli: colored output in pretty print mode
- autokuma: add on_delete config option, see [#29](https://github.com/BigBoot/AutoKuma/issues/29)

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

