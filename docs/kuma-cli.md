---
icon: lucide/terminal
---

# Kuma CLI

Kuma CLI is a command-line tool for managing Uptime Kuma directly from the terminal. It lets you add, edit, delete, and inspect monitors, tags, notifications, maintenances, status pages, and Docker hosts.

## Installation

### Prebuilt Binaries

Download from [GitHub Releases](https://github.com/BigBoot/AutoKuma/releases/latest).

### Snap

```bash
sudo snap install kuma
```

### Cargo

```bash
cargo install --git https://github.com/BigBoot/AutoKuma.git kuma-cli
```

## Usage

```
Usage: kuma [OPTIONS] [COMMAND]

Commands:
  monitor       Manage Monitors
  notification  Manage Notifications
  tag           Manage Tags
  maintenance   Manage Maintenances
  status-page   Manage Status Pages
  docker-host   Manage Docker Hosts
  help          Print this message or the help of the given subcommand

Options:
      --url <URL>
          URL of the Uptime Kuma instance
      --username <USERNAME>
          Uptime Kuma username
      --password <PASSWORD>
          Uptime Kuma password
      --mfa-token <MFA_TOKEN>
          One-time MFA token
      --header <KEY=VALUE>
          Add a custom HTTP header
      --connect-timeout <CONNECT_TIMEOUT>
          Connection timeout [default: 30.0]
      --call-timeout <CALL_TIMEOUT>
          API call timeout [default: 30.0]
      --ready-timeout <READY_TIMEOUT>
          Time to wait for Uptime Kuma to send its initial state after connecting [default: 9.0]
      --format <OUTPUT_FORMAT>
          Output format [default: json] [possible values: json, toml, yaml]
      --pretty
          Pretty-print the output
  -h, --help
          Print help
  -V, --version
          Print version
```

## Available Commands

Each resource type supports a consistent set of subcommands:

| Command | Description |
|---------|-------------|
| `add` | Create a new entity |
| `delete` | Delete an entity |
| `edit` | Edit an existing entity |
| `list` / `ls` | List all entities |
| `get` | Get a single entity by ID |
| `pause` | Pause a monitor or maintenance |
| `resume` | Resume a monitor or maintenance |
| `test` | Test a Docker host connection |

## Configuration

You can set connection details with environment variables instead of passing flags every time:

```bash
KUMA__URL="http://localhost:3001/"
KUMA__USERNAME="admin"
KUMA__PASSWORD="secret"
```

Kuma CLI also reads config files from:

| Platform | Path |
|----------|------|
| Linux | `$XDG_CONFIG_HOME/kuma/config.{toml,yaml,json}` |
| macOS | `$HOME/Library/Application Support/kuma/config.{toml,yaml,json}` |
| Windows | `%LocalAppData%\kuma\config.{toml,yaml,json}` |

### Example TOML Config

```toml
url = "http://localhost:3001/"
username = "admin"
password = "secret"
```

## Examples

```bash
# List all monitors
kuma monitor list

# Get a specific monitor as YAML
kuma --format yaml --pretty monitor get 1

# Add an HTTP monitor from a JSON file
kuma monitor add < monitor.json

# Pause a monitor
kuma monitor pause 1

# Inspect available notification options for a provider
kuma notification get 1
```
