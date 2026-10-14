---
icon: lucide/settings
---

# Configuration

AutoKuma is configured through environment variables or a configuration file. All environment variable names mirror the config file keys with `AUTOKUMA__` as a prefix and `__` as a path separator.

## Environment Variables

| Environment Variable | Config Key | Description |
|---------------------|------------|-------------|
| `AUTOKUMA__STATIC_MONITORS` | `static_monitors` | Path to the folder AutoKuma scans for static monitor definitions |
| `AUTOKUMA__TAG_NAME` | `tag_name` | Name of the AutoKuma tag used to track managed monitors |
| `AUTOKUMA__TAG_COLOR` | `tag_color` | Color of the AutoKuma tag |
| `AUTOKUMA__DEFAULT_SETTINGS` | `default_settings` | Default settings applied to all generated monitors |
| `AUTOKUMA__LOG_DIR` | `log_dir` | Path to a directory where log files are stored |
| `AUTOKUMA__ON_DELETE` | `on_delete` | What happens when an AutoKuma ID is no longer found: `delete` or `keep` |
| `AUTOKUMA__DELETE_GRACE_PERIOD` | `delete_grace_period` | Seconds to wait before deleting an entity after its ID disappears |
| `AUTOKUMA__INSECURE_ENV_ACCESS` | `insecure_env_access` | Allow templates to access all env variables (default: only `AUTOKUMA__ENV__*`) |
| `AUTOKUMA__SNIPPETS__<NAME>` | `snippets.<name>` | Define a snippet named `<name>` |
| `AUTOKUMA__KUMA__URL` | `kuma.url` | URL AutoKuma uses to connect to Uptime Kuma |
| `AUTOKUMA__KUMA__USERNAME` | `kuma.username` | Uptime Kuma username |
| `AUTOKUMA__KUMA__PASSWORD` | `kuma.password` | Uptime Kuma password |
| `AUTOKUMA__KUMA__MFA_TOKEN` | `kuma.mfa_token` | One-time MFA token |
| `AUTOKUMA__KUMA__MFA_SECRET` | `kuma.mfa_secret` | MFA secret for generating tokens automatically |
| `AUTOKUMA__KUMA__AUTH_TOKEN` | `kuma.auth_token` | JWT auth token (alternative to username/password) |
| `AUTOKUMA__KUMA__HEADERS` | `kuma.headers` | Extra HTTP headers sent to Uptime Kuma |
| `AUTOKUMA__KUMA__CONNECT_TIMEOUT` | `kuma.connect_timeout` | Timeout for the initial connection |
| `AUTOKUMA__KUMA__CALL_TIMEOUT` | `kuma.call_timeout` | Timeout for API calls |
| `AUTOKUMA__KUMA__READY_TIMEOUT` | `kuma.ready_timeout` | Seconds to wait after connecting for Uptime Kuma to send its initial state (monitor list etc.) before giving up (default: `9`). Raise it if Kuma is slow to serve logins, e.g. right after a restart with many AutoKuma instances reconnecting |
| `AUTOKUMA__KUMA__TLS__VERIFY` | `kuma.tls.verify` | Whether to verify the TLS certificate |
| `AUTOKUMA__KUMA__TLS__CERT` | `kuma.tls.cert` | Path to a custom PEM certificate |
| `AUTOKUMA__DOCKER__ENABLED` | `docker.enabled` | Enable or disable the Docker source |
| `AUTOKUMA__DOCKER__HOSTS` | `docker.hosts` | Docker host list (string or JSON array) |
| `AUTOKUMA__DOCKER__LABEL_PREFIX` | `docker.label_prefix` | Prefix used when scanning container labels |
| `AUTOKUMA__DOCKER__SOURCE` | `docker.source` | Source for monitor definitions: `Containers`, `Services`, or `Both` |
| `AUTOKUMA__DOCKER__EXCLUDE_CONTAINER_PATTERNS` | `docker.exclude_container_patterns` | Regex patterns to exclude containers by name (semicolon-separated) |
| `AUTOKUMA__KUBERNETES__ENABLED` | `kubernetes.enabled` | Enable or disable the Kubernetes source |
| `AUTOKUMA__FILES__ENABLED` | `files.enabled` | Enable or disable the Files source |
| `AUTOKUMA__FILES__FOLLOW_SYMLINKS` | `files.follow_symlinks` | Follow symlinks when scanning for static monitors |
| `AUTOKUMA__FILES__PREPROCESS` | `files.preprocess` | Preprocess static monitor files as Tera templates before parsing (default: `false`). Allows use of `get_env` and other Tera expressions inside `.json`/`.toml` monitor files. The following filters are available to safely embed values into file contents: `json_escape` / `json_unescape` for JSON files, `toml_escape` / `toml_unescape` for TOML files |

## Secret Files

Any environment variable value starting with `@/` is treated as a file path. AutoKuma reads the file and uses its contents as the value (trailing newline stripped). This is useful for Docker secrets or Kubernetes secrets:

```yaml
AUTOKUMA__KUMA__PASSWORD: "@/run/secrets/kuma_password"
```

To use a literal value that starts with `@/`, escape it with `@@/`:

```yaml
AUTOKUMA__SOME__VALUE: "@@/not_a_file"  # resolved as @/not_a_file
```

## Config File

AutoKuma reads configuration from `autokuma.{toml,yaml,json}` in the current directory and in the following platform-specific locations:

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/autokuma/config.{toml,yaml,json}` |
| macOS | `$HOME/Library/Application Support/autokuma/config.{toml,yaml,json}` |
| Windows | `%LocalAppData%\autokuma\config.{toml,yaml,json}` |

### Example TOML Config

```toml
[kuma]
url = "http://localhost:3001/"
username = "<username>"
password = "<password>"

[kuma.tls]
verify = true
# cert = "/path/to/kuma-ca.pem"

[docker]
enabled = true
source = "both"
label_prefix = "kuma"

[[docker.hosts]]
url = "tcp://docker-a:2376"
tls_verify = true
tls_cert_path = "/certs/docker-a"

[[docker.hosts]]
url = "unix:///var/run/docker.sock"

[files]
enabled = true
follow_symlinks = false
preprocess = false

[kubernetes]
enabled = false
```

## Default Settings

The `default_settings` option lets you apply settings to all monitors of a given type. The format is:

```
<type>.<setting>: <value>
```

You can use `*` as a wildcard for the type:

```yaml
AUTOKUMA__DEFAULT_SETTINGS: |-
  docker.docker_container: {{container_name}}
  http.max_redirects: 10
  *.max_retries: 3
```

## Docker Hosts

The `docker.hosts` option accepts two formats:

```bash
# Semicolon-separated URLs (legacy)
AUTOKUMA__DOCKER__HOSTS="tcp://docker-a:2375;tcp://docker-b:2375"

# JSON array with per-host TLS settings
AUTOKUMA__DOCKER__HOSTS='[{"url":"tcp://docker-a:2376","tls_verify":true,"tls_cert_path":"/certs/docker-a"}]'
```
