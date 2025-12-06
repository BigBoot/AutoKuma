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
| `docker_container`     |               |
| `docker_host`          |               |
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
| `game`                    |               |
| `gamedig_given_port_only` |               |
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
| `description`          | A Monitor     |
| `grpc_body`            |               |
| `grpc_enable_tls`      |               |
| `grpc_metadata`        |               |
| `grpc_method`          |               |
| `grpc_protobuf`        |               |
| `grpc_service_name`    |               |
| `grpc_url`             |               |
| `interval`             | 60            |
| `invert_keyword`       |               |
| `keyword`              |               |
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
| `auth_domain`          |                     |
| `authMethod`           |                     |
| `auth_workstation`     |                     |
| `basic_auth_user`      |                     |
| `basic_auth_pass`      |                     |
| `body`                 |                     |
| `description`          | A Monitor           |
| `expiry_notification`  | true                |
| `http_body_encoding`   |                     |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `name`                 | Example             |
| `oauth_auth_method`    |                     |
| `oauth_client_id`      |                     |
| `oauth_client_secret`  |                     |
| `oauth_scopes`         |                     |
| `oauth_token_url`      |                     |
| `parent`               | 0                   |
| `proxy_id`             |                     |
| `resend_interval`      | 60                  |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `tls_ca`               |                     |
| `tls_cert`             |                     |
| `tls_key`              |                     |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `json-query`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `auth_domain`          |                     |
| `authMethod`           |                     |
| `auth_workstation`     |                     |
| `basic_auth_user`      |                     |
| `basic_auth_pass`      |                     |
| `body`                 |                     |
| `description`          | A Monitor           |
| `expected_value`       |                     |
| `expiry_notification`  | true                |
| `http_body_encoding`   |                     |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `json_path`            |                     |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `name`                 | Example             |
| `oauth_auth_method`    |                     |
| `oauth_client_id`      |                     |
| `oauth_client_secret`  |                     |
| `oauth_scopes`         |                     |
| `oauth_token_url`      |                     |
| `parent`               | 0                   |
| `proxy_id`             |                     |
| `resend_interval`      | 60                  |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `tls_ca`               |                     |
| `tls_cert`             |                     |
| `tls_key`              |                     |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `kafka-producer`
| Property                                             | Example Value |
|------------------------------------------------------|---------------|
| `accepted_statuscodes`                               | 200-299       |
| `active`                                             | true          |
| `description`                                        | A Monitor     |
| `interval`                                           | 60            |
| `kafka_producer_allow_auto_topic_creation`           |               |
| `kafka_producer_brokers`                             |               |
| `kafka_producer_message`                             |               |
| `kafka_producer_ssl`                                 |               |
| `kafka_producer_topic`                               |               |
| `kafka_producer_sasl_options.mechanism`              | plain         |
| `kafka_producer_sasl_options.username`               |               |
| `kafka_producer_sasl_options.password`               |               |
| `kafka_producer_sasl_options.authorization_identity` |               |
| `kafka_producer_sasl_options.access_key_id`          |               |
| `kafka_producer_sasl_options.secret_access_key`      |               |
| `kafka_producer_sasl_options.session_token`          |               |
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
| `auth_domain`          |                     |
| `authMethod`           |                     |
| `auth_workstation`     |                     |
| `basic_auth_user`      |                     |
| `basic_auth_pass`      |                     |
| `body`                 |                     |
| `description`          | A Monitor           |
| `expiry_notification`  | true                |
| `http_body_encoding`   |                     |
| `ignore_tls`           | false               |
| `interval`             | 60                  |
| `invert_keyword`       |                     |
| `keyword`              |                     |
| `max_redirects`        | 10                  |
| `max_retries`          | 0                   |
| `method`               | GET                 |
| `method`               | GET                 |
| `name`                 | Example             |
| `oauth_auth_method`    |                     |
| `oauth_client_id`      |                     |
| `oauth_client_secret`  |                     |
| `oauth_scopes`         |                     |
| `oauth_token_url`      |                     |
| `parent`               | 0                   |
| `proxy_id`             |                     |
| `resend_interval`      | 60                  |
| `retry_interval`       | 60                  |
| `timeout`              | 48                  |
| `tls_ca`               |                     |
| `tls_cert`             |                     |
| `tls_key`              |                     |
| `upside_down`          | false               |
| `url`                  | https://example.com |

## `mongodb`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `mqtt`
| Property               | Example Value |
|------------------------|---------------|
| `active`               | true          |
| `description`          | A Monitor     |
| `interval`             | 60            |
| `hostname`             | localhost     |
| `max_retries`          | 0             |
| `mqtt_check_type`      |               |
| `mqtt_password`        |               |
| `mqtt_success_message` |               |
| `mqtt_topic`           |               |
| `mqtt_username`        |               |
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
| `database_connection_string` |               |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `radius_password`            |               |
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
| `database_connection_string` |               |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
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
| `radius_called_station_id`  |               |
| `radius_calling_station_id` |               |
| `radius_password`           |               |
| `radius_secret`             |               |
| `radius_username`           |               |
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
| `remote_browser`         |                     |
| `remote_browsers_toggle` |                     |
| `retry_interval`         | 60                  |
| `upside_down`            | false               |
| `url`                    | https://example.com |

## `redis`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `description`                | A Monitor     |
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
| `database_connection_string` |               |
| `description`                | A Monitor     |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | false         |

## `tailscale-ping`
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
| `retry_interval`       | 60            |
| `upside_down`          | false         |
