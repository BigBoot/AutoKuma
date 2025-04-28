<div align="center" width="100%">
    <img src="./logo.svg" height="196" alt="" />
</div>

#### [](HEADER)

<div align="center" width="100%">
    <p>
        <a href="https://github.com/BigBoot/AutoKuma/actions"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/BigBoot/AutoKuma/docker-build-push.yml?style=flat&logo=rust&link=https%3A%2F%2Fgithub.com%2FBigBoot%2FAutoKuma%2Factions"></a>
        <a href="https://github.com/BigBoot/AutoKuma/releases/latest"><img alt="GitHub Tag" src="https://img.shields.io/github/v/tag/BigBoot/AutoKuma?logo=github&label=latest"></a>
        <a href="https://ghcr.io/bigboot/autokuma"><img alt="GHCR Tag" src="https://img.shields.io/github/v/tag/BigBoot/AutoKuma?logo=docker&logoColor=white&label=GHCR"></a>
    </p>
    <p>
        <b>
            <a href="#autokuma--">AutoKuma</a>
            &nbsp&nbsp
            <a href="#kuma-cli---">Kuma CLI</a> 
            &nbsp&nbsp
            <a href="#kuma-client--">Kuma Client</a> 
        </b>
    </p>
</div>


# AutoKuma 🐻 <a href="https://crates.io/crates/autokuma"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/autokuma?logo=rust&color=blue"></a>

AutoKuma is a utility that automates the creation of Uptime Kuma monitors based on Docker container labels. With AutoKuma, you can eliminate the need for manual monitor creation in the Uptime Kuma UI.



## How to Install 📦

### Supported Platforms 
| Platform | Architecture | Docker Images | Prebuild Binaries | Notes                            |
|----------|--------------|---------------|-------------------|----------------------------------|
| Windows  | x64          | ⛔︎            | ✅                 |                                  |
| Windows  | arm64        | ⛔︎            | ⛔︎                |                                  |
| Linux    | x64          | ✅             | ✅*                | CLI Snaps Experimental           |
| Linux    | arm64        | ⚠️*           | ⛔︎                | Experimental/Only release builds |
| Mac      | x64          | ⛔︎            | ⛔︎                |                                  |
| Mac      | arm64        | ⛔︎            | ⚠️*               | Experimental                     |


Binaries for windows linux and mac are provided for [GitHub Releases](https://github.com/BigBoot/AutoKuma/releases/latest), additionally AutoKuma is available as a Docker container on [GitHub Container Registry (GHCR)](https://github.com/BigBoot/AutoKuma/pkgs/container/autokuma). To install, simply pull the container using:

Latest Release:

```bash
docker pull ghcr.io/bigboot/autokuma:latest
```

Dev Version:

```bash
docker pull ghcr.io/bigboot/autokuma:master
```


❗ The dev version might break or contain breaking changes without warning, usage on a production system is not adviced.

### Example Docker Compose 🚀

Here's an example `docker-compose.yml`:

```yaml
version: '3'

services:
  autokuma:
    image: ghcr.io/bigboot/autokuma:latest
    restart: unless-stopped
    environment:
      AUTOKUMA__KUMA__URL: http://localhost:3001
      # AUTOKUMA__KUMA__USERNAME: <username> 
      # AUTOKUMA__KUMA__PASSWORD: <password>
      # AUTOKUMA__KUMA__MFA_TOKEN: <token>
      # AUTOKUMA__KUMA__HEADERS: "<header1_key>=<header1_value>,<header2_key>=<header2_value>,..."
      # AUTOKUMA__KUMA__CALL_TIMEOUT: 5
      # AUTOKUMA__KUMA__CONNECT_TIMEOUT: 5
      # AUTOKUMA__TAG_NAME: AutoKuma
      # AUTOKUMA__TAG_COLOR: "#42C0FB"
      # AUTOKUMA__DEFAULT_SETTINGS: |- 
      #    docker.docker_container: {{container_name}}
      #    http.max_redirects: 10
      #    *.max_retries: 3
      # AUTOKUMA__SNIPPETS__WEB: |- 
      #    {{container_name}}_http.http.name: {{container_name}} HTTP
      #    {{container_name}}_http.http.url: https://{{@0}}:{{@1}}
      #    {{container_name}}_docker.docker.name: {{container_name}} Docker
      #    {{container_name}}_docker.docker.docker_container: {{container_name}}
      # AUTOKUMA__DOCKER__HOSTS: unix:///var/run/docker.sock
      # AUTOKUMA__DOCKER__LABEL_PREFIX: kuma
      
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - autokuma-data:/data

volumes:
  autokuma-data:
```

## Configuration 🔧

AutoKuma can be configured using the following environment variables/config keys:

| Env Variable                       | Config Key              | Description                                                                                                              |
|------------------------------------|-------------------------|--------------------------------------------------------------------------------------------------------------------------|
| `AUTOKUMA__STATIC_MONITORS`        | `static_monitors`       | The path to the folder in which AutoKuma will search for static Monitor definitions                                      |
| `AUTOKUMA__TAG_NAME`               | `tag_name`              | The name of the AutoKuma tag, used to track managed containers                                                           |
| `AUTOKUMA__TAG_COLOR`              | `tag_color`             | The color of the AutoKuma tag                                                                                            |
| `AUTOKUMA__DEFAULT_SETTINGS`       | `default_settings`      | Default settings applied to all generated Monitors, see the example above for the syntax                                 |
| `AUTOKUMA__LOG_DIR`                | `log_dir`               | Path to a directory where log files will be stored                                                                       |
| `AUTOKUMA__ON_DELETE`              | `on_delete`             | Specify what should happen to a monitor if the autokuma id is not found anymore, either `delete` or `keep`               |
| `AUTOKUMA__DELETE_GRACE_PERIOD`    | `delete_grace_period`   | How long to wait in seconds before deleting the entity if the autokuma is not not found anymore                          |
| `AUTOKUMA__INSECURE_ENV_ACCESS`    | `insecure_env_access`   | Allow access to all env variables in templates, by default only variables starting with AUTOKUMA__ENV__ can be accessed. |
| `AUTOKUMA__SNIPPETS__<SNIPPET>`    | `snippets.<snippet>`    | Define a snippet named `<snippet>`, see [Snippets](#snippets) for details                                                |
| `AUTOKUMA__KUMA__URL`              | `kuma.url`              | The URL AutoKuma should use to connect to Uptime Kuma                                                                    |
| `AUTOKUMA__KUMA__USERNAME`         | `kuma.username`         | The username for logging into Uptime Kuma (required unless auth is disabled)                                             |
| `AUTOKUMA__KUMA__PASSWORD`         | `kuma.password`         | The password for logging into Uptime Kuma (required unless auth is disabled)                                             |
| `AUTOKUMA__KUMA__MFA_TOKEN`        | `kuma.mfa_token`        | The MFA token for logging into Uptime Kuma (required if MFA is enabled)                                                  |      
| `AUTOKUMA__KUMA__MFA_SECRET`       | `kuma.mfa_secret`       | The MFA secret. Used to generate a tokens for logging into Uptime Kuma (alternative to a single_use mfa_token)           |
| `AUTOKUMA__KUMA__HEADERS`          | `kuma.headers`          | List of HTTP headers to send when connecting to Uptime Kuma                                                              |
| `AUTOKUMA__KUMA__CONNECT_TIMEOUT`  | `kuma.connect_timeout`  | The timeout for the initial connection to Uptime Kuma                                                                    |
| `AUTOKUMA__KUMA__CALL_TIMEOUT`     | `kuma.call_timeout`     | The timeout for executing calls to the Uptime Kuma server                                                                |
| `AUTOKUMA__DOCKER__HOSTS`          | `docker.hosts`          | List of Docker hosts.  Use a semicolon separated string when setting using an env variable.                              |
| `AUTOKUMA__DOCKER__LABEL_PREFIX`   | `docker.label_prefix`   | Prefix used when scanning for container labels                                                                           |
| `AUTOKUMA__DOCKER__SOURCE`         | `docker.source`         | Whether monitors should be created from `Containers` or `Services` labels (or `Both`).                                   |
| `AUTOKUMA__DOCKER__TLS__VERIFY`    | `docker.tls.verify`     | Whether to verify the TLS certificate or not.                                                                            |
| `AUTOKUMA__DOCKER__TLS__CERT`      | `docker.tls.cert`       | The path to a custom tls certificate in PEM format.                                                                      |
| `AUTOKUMA__FILES__FOLLOW_SYMLINKS` | `files.follow_symlinks` | Whether AutoKuma should follow symlinks when looking for "static monitors" (Defaults to false)                           |

AutoKuma will read configuration from a file named `autokuma.{toml,yaml,json}` in the current directory and in the following locations:

| Platform | Value                                                                | Example                                                       |
|----------|----------------------------------------------------------------------|---------------------------------------------------------------|
| Linux    | `$XDG_CONFIG_HOME`/autokuma/config.{toml,yaml,json}                  | /home/alice/.config/autokuma/config.toml                      |
| macOS    | `$HOME`/Library/Application Support/autokuma/config.{toml,yaml,json} | /Users/Alice/Library/Application Support/autokuma/config.toml |
| Windows  | `%LocalAppData%`\autokuma\config.{toml,yaml,json}                    | C:\Users\Alice\AppData\Local\autokuma\config.toml             |

An example `.toml` config could look like the following:
```toml
[kuma]
url = "http://localhost:3001/"
username = "<username>"
password = "<password>"
```


## Usage 💡

AutoKuma interprets Docker container labels with the following format:

```plaintext
<prefix>.<id>.<type>.<setting>: <value>
```

- `<prefix>`: Default is `kuma` unless changed using the `DOCKER__LABEL_PREFIX` env variable.
- `<id>`: A unique identifier for the monitor (ensure it's unique between all monitors).
- `<type>`: The type of the monitor as configured in Uptime Kuma.
- `<setting>`: The key of the value to be set.
- `<value>`: The value for the option.

Labels are grouped by `<id>` into a single monitor. For example, to create a simple HTTP monitor, use the following labels:

```plaintext
kuma.example.http.name: "Example"
kuma.example.http.url: "https://example.com"
```

Take a look at [all available entity types](ENTITY_TYPES.md) and their corresponding settings.

### Groups
To assign a monitor to a group set it's parent_name property to the autokuma id of the group, e.g.

```yaml
kuma.mygroup.group.name: 'This is a Group'

kuma.mymonitor.http.name: 'This is a Monitor assigned to a Group'
kuma.mymonitor.http.parent_name: 'mygroup'
kuma.mymonitor.http.url: 'https://example.com'
```

### Notifications
**_WARNING:_** Defining Notifications is currently experimental and might change in the future.
```yaml
kuma.mynotificationprovider.notification.name: 'This is a Matrix notification provider'
kuma.mynotificationprovider.notification.active: 'true'
kuma.mynotificationprovider.notification.config: '{"type": "matrix", "accessToken": "XXXXXXXXXXXXXXXXXX", "homeserverUrl": "https://matrix.org", "internalRoomId": "!xxxxxxxxxxxxxxxxxx:matrix.org"}'

kuma.mymonitor.http.name: 'This is a Monitor with a notification provider'
kuma.mymonitor.http.notification_names: '["mynotificationprovider"]'
kuma.mymonitor.http.url: 'https://example.com'
```


### Docker Hosts
**_WARNING:_** Defining Docker Hosts is currently experimental and might change in the future.
```yaml
kuma.mydocker.docker_host.name: 'My Docker Host'
kuma.mydocker.docker_host.connection_type: 'socket'
kuma.mydocker.docker_host.path: '/var/run/docker.sock'

kuma.mymonitor.http.name: 'This is a Docker Monitor'
kuma.mymonitor.http.docker_host_name: 'mydocker'
kuma.mymonitor.http.url: 'https://example.com'
```


### Tags
**_WARNING:_** Defining Tags is currently experimental and might change in the future.
```yaml
kuma.mytag.tag.name: 'A purple label'
kuma.mytag.tag.color: '#FF00FF'

kuma.mymonitor.http.name: 'This is a Monitor with a label'
kuma.mymonitor.http.tag_names: '[{"name": "mytag", "value": "A value (this is optional)" }]'
kuma.mymonitor.http.url: 'https://example.com'
```

### Templating
AutoKuma allows the usage of [Tera](https://keats.github.io/tera/) templates in labels and [Snippets](#snippets), the following variables are available:
|                | Template         | Description                             | Example Value                                                                                                                                        |
|----------------|------------------|-----------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Containers** |                  |                                         |                                                                                                                                                      |
|                | `container_id`   | The container id                        | 92366941fb1f211c573c56d261f3b3e5302f354941f2aa295ae56d5781e97221                                                                                     |
|                | `image_id`       | Sha256 of the container image           | sha256:c2e38600b252f147de1df1a5ca7964f9c8e8bace97111e56471a4a431639287a                                                                              |
|                | `image`          | Name of the container image             | ghcr.io/immich-app/immich-server:release                                                                                                             |
|                | `container_name` | Name of the container                   | immich-immich-1                                                                                                                                      |
|                | `container`      | Nested structure with container details | See the [Docker Engine Documentation](https://docs.docker.com/engine/api/v1.45/#tag/Container/operation/ContainerList) for the available data        |
|                | `system_info`    | Nested structure with host details      | See the [Docker Engine Documentation](https://docs.docker.com/reference/api/engine/v1.45/#tag/System/operation/SystemInfo) for the available data    |
| **Services**   |                  |                                         |                                                                                                                                                      |
|                | `service`        | Nested structure with service details   | See the [Docker Engine Documentation](https://docs.docker.com/reference/api/engine/v1.45/#tag/System/operation/ContainerList) for the available data |
|                | `system_info`    | Nested structure with host details      | See the [Docker Engine Documentation](https://docs.docker.com/reference/api/engine/v1.45/#tag/System/operation/SystemInfo) for the available data    |

### Snippets 📝
AutoKuma provides the ability to define reusable snippets. Snippets need to be defined in the configuration, for example, using environment variables:

```yaml
AUTOKUMA__SNIPPETS__WEB: |-
    {{ container_name }}_http.http.name: {{ container_name }} HTTP
    {{ container_name }}_http.http.url: https://{{ args[0] }}:{{ args[1] }}
    {{ container_name }}_docker.docker.name: {{ container_name }} Docker
    {{ container_name }}_docker.docker.docker_container: {{ container_name }}
```

or in an equivalent TOML config file:

```toml
[snippets]
web = '''
    {{ container_name }}_http.http.name: {{ container_name }}
    {{ container_name }}_http.http.url: https://{{ args[0] }}:{{ args[1] }}
    {{ container_name }}_docker.docker.name: {{ container_name }}_docker
    {{ container_name }}_docker.docker.docker_name: {{ container_name }}
'''
```

These define a snippet called `web`. 

A snippet can have a variable number of arguments, which are available as replacements using `{{ args[0] }}`, `{{ args[1] }}`, `{{ args[2] }}`, etc., as seen above.

To use a snippet on a container, assign a label in the format:

```plaintext
<prefix>.__<snippet>: <arguments>
```

For example, the above snippet could be included using the following label:

```plaintext
kuma.__web: '"example.com", 443'
```

Snippets also use [Tera](https://keats.github.io/tera/), which allows for some quite advanced templates, here's a extended variation of the above example:
```jinja
{# Assign the first snippet arg to args to make access easier #}
{% set args = args[0] %}

{# Generate an autokuma id by slugifying the "name" arg #}
{% set id = args.name | slugify %}

{# if we have a "keyword" generate a "keyword" monitor, otherwise generate a "http" monitor #}
{% if args.keyword %}
    {% set type = "keyword" %}
{% else %}
    {% set type = "http" %}
{% endif %}


{# below are the actual lines which end up defining the monitor #}
{{ id }}-group.group.name: {{ args.name }}
{{ id }}-http.{{ type }}.name: {{ args.name }} (HTTP)
{{ id }}-http.{{ type }}.parent_name: {{ id }}-group
{{ id }}-http.{{ type }}.url: {{ args.url }}
{% if args.keyword %}
    {{ id }}-http.{{ type }}.keyword: {{ args.keyword }}
{% endif %}
{% if args.status_code %}
    {{ id }}-http.{{ type }}.status_code: {{ args.status_code }}
{% endif %}
{{ id }}-http-container.docker.name: {{ args.name }} (Container)
{{ id }}-http-container.docker.parent_name: {{ id }}-group
```

And the usage of it would be like the following:
Just a basic http monitor:  
```yaml
kuma.__web: '{ "name": "Example HTTP", "url": "https://example.com" }'
```

Keyword monitor with custom status_codes:
```yaml
kuma.__web: '{ "name": "Example HTTP", "url": "https://example.com", "keyword": "Example Domain", "status_codes": ["200"] }'
```

#### !Snippets
There's a special case for snippets starting with a `!`, these snippets will apply to labels without requiring the prefix (i.e. `kuma.__`). The purpose of these is to be able to reuse existing labels from other tools. (Note: Due to this !Snippets will always receive a single string argument containing the label value instead of a structured list). 

For example you could create a snippet to reuse traefik labels by defining a snippet called `!traefik.enable`:
```jinja
{# Only apply if value is "true" #}
{% if args[0] == "true" %}
    {# Extract some information from existing labels, note that this will likely not fit your setup and you will need to adjust this to get the required data #}
    {% set traefik_service = container_name %}
    {% set domain = container_name + ".example.com" %}
    {% set port = container["Labels"]["traefik.http.services." + traefik_service + ".loadbalancer.server.port"] %}

    {{ container_name }}_http.http.name: {{ container_name }}
    {{ container_name }}_http.http.url: https://{{ domain }}:{{ port }}
{% endif %}
```


### Static Monitors 📊
In addition to reading Monitors from Docker labels, AutoKuma can create Monitors from files. This can be usefull if you have want AutoKuma to manage monitors which aren't directly related to a container.

To create static Monitors just add a .json or .toml file in the directory specified by `AUTOKUMA__STATIC_MONITORS`, take a look at [the examples here](monitors).

The default directory for static monitors is:

| Platform | Value                                                         | Example                                                            |
|----------|---------------------------------------------------------------|--------------------------------------------------------------------|
| Linux    | `$XDG_CONFIG_HOME`/autokuma/static-monitors/                  | /home/alice/.config/autokuma/static-monitors/                      |
| macOS    | `$HOME`/Library/Application Support/autokuma/static-monitors/ | /Users/Alice/Library/Application Support/autokuma/static-monitors/ |
| Windows  | `%LocalAppData%`\autokuma\static-monitors\                    | C:\Users\Alice\AppData\Local\autokuma\static-monitors\             |

In case of static Monitors the id is determined by the filename (without the extension).


# Kuma CLI 🤖 <a href="https://crates.io/crates/kuma-cli"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/kuma-cli?logo=rust&color=blue"></a> [![kuma](https://snapcraft.io/kuma/badge.svg)](https://snapcraft.io/kuma)

Kuma CLI is a Command Line Interface (CLI) tool for managing and interacting with [Uptime Kuma](https://uptime.kuma.pet/). With Kuma CLI you can easily configure, monitor and manage your applications from the command line.

## Features 🎯
- [x] Commands: `kuma monitor`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `list`
    - [x] `get`
    - [x] `pause`
    - [x] `resume`
- [x] Commands : `kuma tag`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `ls`
    - [x] `get`
- [x] Commands : `kuma notification`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `ls`
    - [x] `get`
- [x] Commands : `kuma maintenance`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `ls`
    - [x] `get`
    - [x] `pause`
    - [x] `resume`
- [x] Commands : `kuma status-page`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `ls`
    - [x] `get`
- [x] Commands : `kuma docker-host`
    - [x] `add`
    - [x] `delete`
    - [x] `edit`
    - [x] `ls`
    - [x] `get`
    - [x] `test`

## How to Install 📦

Binaries for Windows, Linux and Mac OS are provided for [GitHub Releases](https://github.com/BigBoot/AutoKuma/releases/latest) and additionally Kuma CLI can be installed using `cargo`:

```bash
cargo install --git https://github.com/BigBoot/AutoKuma.git kuma-cli
```

## Usage 💡

```bash
Usage: kuma [OPTIONS] [COMMAND]

Commands:
  monitor       Manage Monitors
  notification  Manage Notifications
  tag           Manage Tags
  maintenanc    Manage Maintenances
  help          Print this message or the help of the given subcommand(s)

Options:
      --url <URL>
          The URL AutoKuma should use to connect to Uptime Kuma
      --username <USERNAME>
          The username for logging into Uptime Kuma (required unless auth is disabled)
      --password <PASSWORD>
          The password for logging into Uptime Kuma (required unless auth is disabled)
      --mfa-token <MFA_TOKEN>
          The MFA token for logging into Uptime Kuma (required if MFA is enabled)
      --header <KEY=VALUE>
          Add a HTTP header when connecting to Uptime Kuma
      --connect-timeout <CONNECT_TIMEOUT>
          The timeout for the initial connection to Uptime Kuma [default: 30.0]
      --call-timeout <CALL_TIMEOUT>
          The timeout for executing calls to the Uptime Kuma server [default: 30.0]
      --format <OUTPUT_FORMAT>
          The output format [default: json] [possible values: json, toml, yaml]
      --pretty
          Whether the output should be pretty printed or condensed
  -h, --help
          Print help
  -V, --version
          Print version
```


## Configuration 🔧

All configuration options can also be specified as environment variables:
```
KUMA__URL="http://localhost:3001/"
KUMA__USERNAME="<username>"
KUMA__PASSWORD="<password>"
...
```

Additionally Kuma CLI will read configuration from a file named `kuma.{toml,yaml,json}` in the current directory and in the following locations:
| Platform | Value                                                            | Example                                                   |
|----------|------------------------------------------------------------------|-----------------------------------------------------------|
| Linux    | `$XDG_CONFIG_HOME`/kuma/config.{toml,yaml,json}                  | /home/alice/.config/kuma/config.toml                      |
| macOS    | `$HOME`/Library/Application Support/kuma/config.{toml,yaml,json} | /Users/Alice/Library/Application Support/kuma/config.toml |
| Windows  | `%LocalAppData%`\kuma\config.{toml,yaml,json}                    | C:\Users\Alice\AppData\Local\kuma\config.toml             |

An example `.toml` config could look like the following:
```toml
url = "http://localhost:3001/"
username = "<username>"
password = "<password>"
```

# Kuma Client 🧑‍💻 <a href="https://crates.io/crates/kuma-client"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/kuma-client?logo=rust&color=blue"></a>

`kuma-client` is a Rust crate that provides a client library for interacting with the Uptime Kuma SocketIO API.

Please take a look at [the examples](kuma-client/examples/) and the [documentation](https://docs.rs/kuma-client/latest/kuma_client/) for further details.

# Contributing 👥

Contributions to AutoKuma are welcome! Feel free to open issues, submit pull requests, or provide feedback.

# License 📜

AutoKuma is released under the [MIT License](LICENSE).
