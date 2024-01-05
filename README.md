<div align="center" width="100%">
    <img src="./logo.svg" height="196" alt="" />
</div>


# AutoKuma

AutoKuma is a utility that automates the creation of Uptime Kuma monitors based on Docker container labels. With AutoKuma, you can eliminate the need for manual monitor creation in the Uptime Kuma UI.

## 🔧 How to Install

The AutoKuma Docker container is available on [GitHub Container Registry (GHCR)](https://ghcr.io/username/autokuma). To install, simply pull the container using:

```bash
docker pull ghcr.io/bigboot/autokuma:master
```

## Example Docker Compose

Here's an example `docker-compose.yml`:

```yaml
version: '3'

services:
  autokuma:
    image: ghcr.io/bigboot/autokuma:master
    environment:
      - AUTOKUMA__KUMA__URL=http://localhost:3001
      - AUTOKUMA__KUMA__USERNAME=<username>
      - AUTOKUMA__KUMA__PASSWORD=<password>
      - AUTOKUMA__KUMA__MFA_TOKEN=<token>
      - AUTOKUMA__KUMA__HEADERS="<header1_key>=<header1_value>, <header2_key>=<header2_value>, ..."
      - AUTOKUMA__KUMA__TAG_NAME=AutoKuma
      - AUTOKUMA__KUMA__TAG_COLOR=#42C0FB
      - AUTOKUMA__DOCKER__SOCKET=/var/run/docker.sock
      - AUTOKUMA__DOCKER__LABEL_PREFIX=kuma
    labels:
      - "kuma.example.http.name=Example"
      - "kuma.example.http.url=https://example.com"
```

## Configuration

AutoKuma can be configured using the following environment variables:

| Variable                            | Description                                      |
| ----------------------------------- | ------------------------------------------------ |
| `AUTOKUMA__KUMA__URL`               | The url AutoKuma should use to connect to Uptime Kuma |
| `AUTOKUMA__KUMA__USERNAME`          | The username for logging into Uptime Kuma (required unless auth is disabled) |
| `AUTOKUMA__KUMA__PASSWORD`          | The password for logging into Uptime Kuma (required unless auth is disabled) |
| `AUTOKUMA__KUMA__MFA_TOKEN`         | The MFA token for logging into Uptime Kuma (required if MFA is enabled) |
| `AUTOKUMA__KUMA__HEADERS`           | List of HTTP headers used when connecting to Uptime Kuma |
| `AUTOKUMA__KUMA__TAG_NAME`          | The name of the AutoKuma tag, used to track managed containers |
| `AUTOKUMA__KUMA__TAG_COLOR`         | The color of the AutoKuma tag |
| `AUTOKUMA__DOCKER__SOCKET`          | Path to the Docker socket |
| `AUTOKUMA__DOCKER__LABEL_PREFIX`    | Prefix used when scanning for container labels |



## Usage

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

### Example Labels

```plaintext
kuma.example.http.name: "Example"
kuma.example.http.url: "https://example.com"
```

Take a look at [all available monitor types](MONITOR_TYPES.md) and the corresponding settings.


## Contributing

Contributions to AutoKuma are welcome! Feel free to open issues, submit pull requests, or provide feedback.

## License

AutoKuma is released under the [MIT License](LICENSE).