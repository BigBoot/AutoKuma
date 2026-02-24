# AutoKuma specific properties:

AutoKuma adds a few special properties which are handled internally and aren't sent to Uptime Kuma:
| Property                 | Example Value                              | Description                                                                           |
|--------------------------|--------------------------------------------|---------------------------------------------------------------------------------------|
| `parent_name`            | `apps`                                     | The autokuma id of the parent group                                                   |
| `notification_name_list` | `["matrix", "discord"]`                    | List of autokuma ids of enabled notification providers,                               |
| `tag_names`              | `[{"name": "mytag", "value": "A value" }]` | List of structs containing the id and optionally a values for labels,                 |
| `docker_host_name`       | `local_socket`                             | The autokuma id of the docker socket for a docker monitor                             |
| `create_paused`          | false                                      | If true new monitors will be added in paused state, does not effect existing monitors |

# `docker_host`
| Property          | Example Value          |
|-------------------|------------------------|
| `connection_type` | `socket` or `tcp`      |
| `host` or `path`  | `/var/run/docker.sock` |

# `notification`
| Property     | Example Value                                                                                                                                                                     |
|--------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `active`     | `true`                                                                                                                                                                            |
| `is_default` | `true` (Note: this is only used by the WebUI, AutoKuma does not respect this setting for technical reasons)                                                                       |
| `config`     | nested provider specific settings.  Too many to list here. I suggest creating a notification with your provider in the WebUI and then using the `kuma` CLI to inspect the options |

# Monitor Types
- [AutoKuma specific properties:](#autokuma-specific-properties)
- [`docker_host`](#docker_host)
- [`notification`](#notification)
- [Monitor Types](#monitor-types)
  - [`dns`](#dns)
  - [`docker`](#docker)
  - [`gamedig`](#gamedig)
  - [`group`](#group)
  - [`grpc-keyword`](#grpc-keyword)
  - [`http`](#http)
  - [`json-query`](#json-query)
  - [`kafka-producer`](#kafka-producer)
  - [`keyword`](#keyword)
  - [`mongodb`](#mongodb)
  - [`mqtt`](#mqtt)
  - [`mysql`](#mysql)
  - [`ping`](#ping)
  - [`port`](#port)
  - [`postgres`](#postgres)
  - [`push`](#push)
  - [`radius`](#radius)
  - [`real-browser`](#real-browser)
  - [`redis`](#redis)
  - [`steam`](#steam)
  - [`sqlserver`](#sqlserver)
  - [`tailscale-ping`](#tailscale-ping)
  - [`smtp`](#smtp)
  - [`snmp`](#snmp)
  - [`rabbitmq`](#rabbitmq)


## `dns`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `dns_resolve_server`   | 1.1.1.1       |
| `dns_resolve_type`     | A             |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `docker`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `docker_container`     | nginx         |
| `docker_host`          | 1             |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `gamedig`
| Property                  | Example Value |
|---------------------------|---------------|
| `accepted_statuscodes`    | 200-299       |
| `active`                  | true          |
| `description`             | A Monitor     |
| `game`                    | minecraft     |
| `gamedig_given_port_only` | false         |
| `hostname`                | localhost     |
| `interval`                | 60            |
| `max_retries`             | 0             |
| `name`                    | Example       |
| `parent`                  | 0             |
| `port`                    | 0             |
| `retry_interval`          | 60            |
| `upside_down`             | false         |

## `group`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `grpc-keyword`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `cache_bust`           | false         |
| `description`          | A Monitor     |
| `grpc_body`            | {}            |
| `grpc_enable_tls`      | false         |
| `grpc_metadata`        | {"authorization":"Bearer token"} |
| `grpc_method`          | Check         |
| `grpc_protobuf`        | health.proto  |
| `grpc_service_name`    | grpc.health.v1.Health |
| `grpc_url`             | localhost:50051 |
| `interval`             | 60            |
| `invert_keyword`       | false         |
| `keyword`              | healthy       |
| `max_retries`          | 0             |
| `max_redirects`        | 10            |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `http`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `authMethod`           | basic               |
| `basic_auth_user`      | monitor             |
| `basic_auth_pass`      | secret              |
| `oauth_auth_method`    | client_secret_basic |
| `oauth_client_id`      | monitor-client      |
| `oauth_token_url`      | https://auth.example.com/oauth/token |
| `oauth_client_secret`  | secret              |
| `oauth_scopes`         | uptime.read         |
| `auth_domain`          | EXAMPLE             |
| `auth_workstation`     | WORKSTATION1        |
| `tls_cert`             | -----BEGIN CERTIFICATE-----... |
| `tls_key`              | -----BEGIN PRIVATE KEY-----... |
| `tls_ca`               | -----BEGIN CERTIFICATE-----... |
| `body`                 | {"status":"ok"} |
| `cache_bust`           | false               |
| `description`          | A Monitor           |
| `expiry_notification`  | true                |
| `headers`              | {"X-Api-Key":"secret"} |
| `http_body_encoding`   | json                |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `name`                 | Example             |
| `parent`               | 0                   |
| `proxy_id`             | 1                   |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `json-query`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `authMethod`           | basic               |
| `basic_auth_user`      | monitor             |
| `basic_auth_pass`      | secret              |
| `oauth_auth_method`    | client_secret_basic |
| `oauth_client_id`      | monitor-client      |
| `oauth_token_url`      | https://auth.example.com/oauth/token |
| `oauth_client_secret`  | secret              |
| `oauth_scopes`         | uptime.read         |
| `auth_domain`          | EXAMPLE             |
| `auth_workstation`     | WORKSTATION1        |
| `tls_cert`             | -----BEGIN CERTIFICATE-----... |
| `tls_key`              | -----BEGIN PRIVATE KEY-----... |
| `tls_ca`               | -----BEGIN CERTIFICATE-----... |
| `body`                 | {"status":"ok"} |
| `grpc_metadata`        | false               |
| `description`          | A Monitor           |
| `expected_value`       | up                  |
| `expiry_notification`  | true                |
| `headers`              | {"X-Api-Key":"secret"} |
| `http_body_encoding`   | json                |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `json_path`            | $.status            |
| `json_path_operator`   | eq                  |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `name`                 | Example             |
| `parent`               | 0                   |
| `proxy_id`             | 1                   |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `kafka-producer`
| Property                                             | Example Value |
|------------------------------------------------------|---------------|
| `accepted_statuscodes`                               | 200-299       |
| `active`                                             | true          |
| `description`                                        | A Monitor     |
| `interval`                                           | 60            |
| `kafka_producer_allow_auto_topic_creation`           | true          |
| `kafka_producer_brokers`                             | ["localhost:9092"] |
| `kafka_producer_message`                             | autokuma test message |
| `kafka_producer_sasl_options`                        | {"mechanism":"plain","username":"kafka-user","password":"kafka-pass"} |
| `kafka_producer_ssl`                                 | false         |
| `kafka_producer_topic`                               | monitor-events |
| `max_retries`                                        | 0             |
| `name`                                               | Example       |
| `parent`                                             | 0             |
| `retry_interval`                                     | 60            |
| `upside_down`                                        | false         |

## `keyword`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `authMethod`           | basic               |
| `basic_auth_user`      | monitor             |
| `basic_auth_pass`      | secret              |
| `oauth_auth_method`    | client_secret_basic |
| `oauth_client_id`      | monitor-client      |
| `oauth_token_url`      | https://auth.example.com/oauth/token |
| `oauth_client_secret`  | secret              |
| `oauth_scopes`         | uptime.read         |
| `auth_domain`          | EXAMPLE             |
| `auth_workstation`     | WORKSTATION1        |
| `tls_cert`             | -----BEGIN CERTIFICATE-----... |
| `tls_key`              | -----BEGIN PRIVATE KEY-----... |
| `tls_ca`               | -----BEGIN CERTIFICATE-----... |
| `body`                 | {"status":"ok"} |
| `description`          | A Monitor           |
| `expiry_notification`  | true                |
| `headers`              | {"X-Api-Key":"secret"} |
| `http_body_encoding`   | json                |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `invert_keyword`       | false               |
| `keyword`              | healthy             |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `name`                 | Example             |
| `parent`               | 0                   |
| `proxy_id`             | 1                   |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `mongodb`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `command`                    | {"ping":1}   |
| `database_connection_string` | mongodb://localhost:27017/admin |
| `description`                | A Monitor     |
| `expected_value`             | 1             |
| `interval`                   | 60            |
| `json_path`                  | $.ok          |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `mqtt`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `expected_value`       | online        |
| `interval`             | 60            |
| `hostname`             | localhost     |
| `json_path`            | $.status      |
| `json_path_operator`   | eq            |
| `max_retries`          | 0             |
| `mqtt_check_type`      | keyword       |
| `mqtt_password`        | mqtt-pass     |
| `mqtt_success_message` | online        |
| `mqtt_topic`           | sensors/status |
| `mqtt_username`        | mqtt-user     |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `mysql`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` | mysql://root:password@localhost:3306/mysql |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `radius_password`            | secret        |
| `query`                      | SELECT 1      |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `ping`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `packet_size`          | 56            |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `port`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `postgres`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` | postgres://postgres:password@localhost:5432/postgres |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `query`                      | SELECT 1      |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `push`
| Property               | Example Value                    |
|------------------------|----------------------------------|
| `accepted_statuscodes` | 200-299                          |
| `active`               | true                             |
| `description`          | A Monitor                        |
| `interval`             | 60                               |
| `max_retries`          | 0                                |
| `name`                 | Example                          |
| `parent`               | 0                                |
| `push_token`           | 4Gdp9cHeNu7MHZ6P6RPiiVbHgSdEHJz7 |
| `retry_interval`       | 60                               |
| `upside_down`          | false                            |

## `radius`
| Property                    | Example Value |
|-----------------------------|---------------|
| `accepted_statuscodes`      | 200-299       |
| `active`                    | true          |
| `description`               | A Monitor     |
| `hostname`                  | localhost     |
| `interval`                  | 60            |
| `max_retries`               | 0             |
| `name`                      | Example       |
| `parent`                    | 0             |
| `port`                      | 0             |
| `radius_called_station_id`  | AP-01         |
| `radius_calling_station_id` | client-01     |
| `radius_password`           | password      |
| `radius_secret`             | secret        |
| `radius_username`           | monitor       |
| `retry_interval`            | 60            |
| `upside_down`               | false         |

## `real-browser`
| Property                 | Example Value       |
|--------------------------|---------------------|
| `accepted_statuscodes`   | 200-299             |
| `active`                 | true                |
| `description`            | A Monitor           |
| `interval`               | 60                  |
| `max_retries`            | 0                   |
| `name`                   | Example             |
| `parent`                 | 0                   |
| `remote_browser`         | ws://localhost:3000 |
| `remote_browsers_toggle` | true                |
| `retry_interval`         | 60                  |
| `upside_down`            | false               |
| `url`                    | https://example.com |

## `redis`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` | redis://localhost:6379 |
| `description`                | A Monitor     |
| `ignore_tls`                 | false         |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `steam`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `sqlserver`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` | Server=localhost;Database=master;User Id=sa;Password=Password123!; |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `query`                      | SELECT 1      |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `tailscale-ping`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `smtp`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 25            |
| `security`             | none          |
| `retry_interval`       | 60            |
| `upside_down`          | false         |

## `snmp`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `expected_value`       | 1             |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `json_path`            | $.value       |
| `json_path_operator`   | eq            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `oid`                  | 1.3.6.1.2.1.1.3.0 |
| `parent`               | 0             |
| `radius_password`      | public        |
| `port`                 | 161           |
| `retry_interval`       | 60            |
| `upside_down`          | false         |
| `version`              | v2c           |

## `rabbitmq`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `description`          | A Monitor     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `nodes`                | ["http://localhost:15672"] |
| `parent`               | 0             |
| `password`             | guest         |
| `retry_interval`       | 60            |
| `upside_down`          | false         |
| `username`             | guest         |
