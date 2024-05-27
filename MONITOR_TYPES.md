# Monitor Types
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
| `dns_resolve_server`   | 1.1.1.1       |
| `dns_resolve_type`     | A             |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `docker`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `docker_container`     |               |
| `docker_host`          |               |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `gamedig`
| Property                  | Example Value |
|---------------------------|---------------|
| `accepted_statuscodes`    | 200-299       |
| `active`                  | true          |
| `game`                    |               |
| `gamedig_given_port_only` |               |
| `hostname`                | localhost     |
| `interval`                | 60            |
| `max_retries`             | 0             |
| `name`                    | Example       |
| `parent`                  | 0             |
| `port`                    | 0             |
| `retry_interval`          | 60            |
| `upside_down`             | bool          |

## `group`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `grpc-keyword`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
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
| `upside_down`          | bool          |

## `http`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `auth_domain`          |                     |
| `authMethod`           |                     |
| `auth_workstation`     |                     |
| `basic_auth_user`      |                     |
| `basic_auth_password`  |                     |
| `body`                 |                     |
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
| `upside_down`          | bool                |
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
| `basic_auth_password`  |                     |
| `body`                 |                     |
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
| `upside_down`          | bool                |
| `url`                  | https://example.com |

## `kafka-producer`
| Property                                             | Example Value |
|------------------------------------------------------|---------------|
| `accepted_statuscodes`                               | 200-299       |
| `active`                                             | true          |
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
| `upside_down`                                        | bool          |

## `keyword`
| Property               | Example Value       |
|------------------------|---------------------|
| `accepted_statuscodes` | 200-299             |
| `active`               | true                |
| `auth_domain`          |                     |
| `authMethod`           |                     |
| `auth_workstation`     |                     |
| `basic_auth_user`      |                     |
| `basic_auth_password`  |                     |
| `body`                 |                     |
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
| `upside_down`          | bool                |
| `url`                  | https://example.com |

## `mongodb`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | bool          |

## `mqtt`
| Property               | Example Value |
|------------------------|---------------|
| `active`               | true          |
| `hostname`             | localhost     |
| `interval`             | 60            |
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
| `upside_down`          | bool          |

## `mysql`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `radius_password`            |               |
| `retry_interval`             | 60            |
| `upside_down`                | bool          |

## `ping`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `packet_size`          | 56            |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `port`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `postgres`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | bool          |

## `push`
| Property               | Example Value                    |
|------------------------|----------------------------------|
| `accepted_statuscodes` | 200-299                          |
| `active`               | true                             |
| `interval`             | 60                               |
| `max_retries`          | 0                                |
| `name`                 | Example                          |
| `parent`               | 0                                |
| `push_token`           | 4Gdp9cHeNu7MHZ6P6RPiiVbHgSdEHJz7 |
| `retry_interval`       | 60                               |
| `upside_down`          | bool                             |

## `radius`
| Property                    | Example Value |
|-----------------------------|---------------|
| `accepted_statuscodes`      | 200-299       |
| `active`                    | true          |
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
| `upside_down`               | bool          |

## `real-browser`
| Property                 | Example Value       |
|--------------------------|---------------------|
| `accepted_statuscodes`   | 200-299             |
| `active`                 | true                |
| `interval`               | 60                  |
| `max_retries`            | 0                   |
| `name`                   | Example             |
| `parent`                 | 0                   |
| `remote_browser`         |                     |
| `remote_browsers_toggle` |                     |
| `retry_interval`         | 60                  |
| `upside_down`            | bool                |
| `url`                    | https://example.com |

## `redis`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | bool          |

## `steam`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `port`                 | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |

## `sqlserver`
| Property                     | Example Value |
|------------------------------|---------------|
| `accepted_statuscodes`       | 200-299       |
| `active`                     | true          |
| `database_connection_string` |               |
| `interval`                   | 60            |
| `max_retries`                | 0             |
| `name`                       | Example       |
| `parent`                     | 0             |
| `retry_interval`             | 60            |
| `upside_down`                | bool          |

## `tailscale-ping`
| Property               | Example Value |
|------------------------|---------------|
| `accepted_statuscodes` | 200-299       |
| `active`               | true          |
| `hostname`             | localhost     |
| `interval`             | 60            |
| `max_retries`          | 0             |
| `name`                 | Example       |
| `parent`               | 0             |
| `retry_interval`       | 60            |
| `upside_down`          | bool          |