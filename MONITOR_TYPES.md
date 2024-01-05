## `ping`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `packet_size` | 56          |
| `accepted_statuscodes` | 200-299 |

## `mqtt`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `port`   | 0             |
| `mqtt_username` | null     |
| `mqtt_password` | null     |
| `mqtt_topic` | ""          |
| `mqtt_check_type` | null    |
| `mqtt_success_message` | null |

## `redis`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `database_connection_string` | null |
| `accepted_statuscodes` | 200-299 |

## `push`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `push_url` | null         |
| `accepted_statuscodes` | 200-299 |

## `mysql`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `database_connection_string` | null |
| `radius_password` | ""      |
| `accepted_statuscodes` | 200-299 |

## `docker`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `docker_container` | ""  |
| `docker_host` | ""       |
| `accepted_statuscodes` | 200-299 |

## `tailscale-ping`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `accepted_statuscodes` | 200-299 |

## `radius`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `port`   | 0             |
| `radius_username` | ""     |
| `radius_password` | ""     |
| `radius_secret` | ""       |
| `radius_called_station_id` | "" |
| `radius_calling_station_id` | "" |
| `accepted_statuscodes` | 200-299 |

## `kafka-producer`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `kafka_producer_brokers` | null |
| `kafka_producer_topic` | null |
| `kafka_producer_message` | null |
| `kafka_producer_ssl` | null |
| `kafka_producer_allow_auto_topic_creation` | null |
| `accepted_statuscodes` | 200-299 |

## `gamedig`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `game`   | ""            |
| `hostname` | localhost    |
| `port`   | 0             |
| `gamedig_given_port_only` | null |
| `accepted_statuscodes` | 200-299 |

## `real-browser`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `url`    | https://example.com |
| `remote_browsers_toggle` | null |
| `remote_browser` | null   |
| `accepted_statuscodes` | 200-299 |

## `sqlserver`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `database_connection_string` | null |
| `accepted_statuscodes` | 200-299 |

## `group`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `accepted_statuscodes` | 200-299 |

## `http`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `url`    | https://example.com |
| `timeout` | 48            |
| `method` | GET           |
| `accepted_statuscodes` | 200-299 |

## `grpc-keyword`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `keyword` | ""            |
| `invert_keyword` | null    |
| `grpc_url` | ""            |
| `maxredirects` | 10       |
| `grpc_enable_tls` | null   |
| `grpc_service_name` | ""    |
| `grpc_method` | ""         |
| `grpc_protobuf` | null     |
| `grpc_body` | null         |
| `grpc_metadata` | null     |
| `accepted_statuscodes` | 200-299 |

## `mongodb`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `database_connection_string` | null |
| `accepted_statuscodes` | 200-299 |

## `keyword`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `url`    | https://example.com |
| `timeout` | 48            |
| `method` | GET           |
| `keyword` | ""            |
| `invert_keyword` | null    |
| `accepted_statuscodes` | 200-299 |

## `json-query`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `url`    | https://example.com |
| `timeout` | 48            |
| `json_path` | null         |
| `expected_Example value` | null   

 |
| `accepted_statuscodes` | 200-299 |

## `steam`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `port`   | 0             |
| `accepted_statuscodes` | 200-299 |

## `dns`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `dns_resolve_server` | 1.1.1.1 |
| `port`   | 0             |
| `dns_resolve_type` | A   |
| `accepted_statuscodes` | 200-299 |

## `port`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `hostname` | localhost    |
| `port`   | 0             |
| `accepted_statuscodes` | 200-299 |

## `postgres`
| Property | Example Value         |
|----------|---------------|
| `name`   | Example       |
| `interval` | 60           |
| `database_connection_string` | null |
| `accepted_statuscodes` | 200-299 |