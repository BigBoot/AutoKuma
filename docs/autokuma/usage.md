---
icon: lucide/play
---

# Usage

## Label Format

AutoKuma reads Docker container labels in the following format:

```
<prefix>.<id>.<type>.<setting>: <value>
```

- `<prefix>` - Default is `kuma`. Change it with `AUTOKUMA__DOCKER__LABEL_PREFIX`.
- `<id>` - A unique identifier for the monitor. Must be unique across all monitors.
- `<type>` - The monitor type as configured in Uptime Kuma (e.g. `http`, `ping`, `dns`).
- `<setting>` - The property name to set.
- `<value>` - The value.

Labels sharing the same `<id>` are merged into a single monitor definition.

### Basic Example

```yaml
labels:
  kuma.example.http.name: "Example"
  kuma.example.http.url: "https://example.com"
```

This creates an HTTP monitor named `Example` pointing at `https://example.com`.

See [Entity Types](../entity-types/index.md) for all available monitor types and their properties.

## Groups

To assign a monitor to a group, set its `parent_name` property to the AutoKuma ID of the group:

```yaml
labels:
  kuma.mygroup.group.name: "This is a Group"
  kuma.mymonitor.http.name: "This is a Monitor assigned to a Group"
  kuma.mymonitor.http.parent_name: "mygroup"
  kuma.mymonitor.http.url: "https://example.com"
```

## Notifications

!!! warning "Experimental"
    Defining notifications is experimental and may change in future releases.

```yaml
labels:
  kuma.mynotif.notification.name: "Matrix"
  kuma.mynotif.notification.active: "true"
  kuma.mynotif.notification.config: >-
    {"type": "matrix", "accessToken": "XXXXXXXXXX",
     "homeserverUrl": "https://matrix.org",
     "internalRoomId": "!xxxxxxxxxx:matrix.org"}

  kuma.mymonitor.http.name: "My Monitor"
  kuma.mymonitor.http.notification_name_list: '["mynotif"]'
  kuma.mymonitor.http.url: "https://example.com"
```

## Docker Hosts

!!! warning "Experimental"
    Defining Docker hosts via labels is experimental and may change in future releases.

```yaml
labels:
  kuma.mydocker.docker_host.name: "My Docker Host"
  kuma.mydocker.docker_host.connection_type: "socket"
  kuma.mydocker.docker_host.path: "/var/run/docker.sock"

  kuma.mymonitor.docker.name: "My Container Monitor"
  kuma.mymonitor.docker.docker_host_name: "mydocker"
  kuma.mymonitor.docker.docker_container: "my-container"
```

## Tags

!!! warning "Experimental"
    Defining tags via labels is experimental and may change in future releases.

```yaml
labels:
  kuma.mytag.tag.name: "A purple label"
  kuma.mytag.tag.color: "#FF00FF"

  kuma.mymonitor.http.name: "My Monitor"
  kuma.mymonitor.http.tag_names: '[{"name": "mytag", "value": "optional value"}]'
  kuma.mymonitor.http.url: "https://example.com"
```

## Container Exclusion

AutoKuma can exclude containers by name using regular expressions. This is useful for filtering out temporary containers created during Docker Compose stack updates.

```yaml
environment:
  # Exclude Docker Compose temporary containers
  AUTOKUMA__DOCKER__EXCLUDE_CONTAINER_PATTERNS: "^[a-f0-9]{12}_.*_"
```

Patterns are separated by semicolons:

| Pattern | Use Case |
|---------|----------|
| `^[a-f0-9]{12}_.*_` | Docker Compose temporary containers |
| `_test$\|test_` | Containers with "test" in the name |
| `^temp_;^[a-f0-9]{12}_.*_` | Multiple patterns |

Exclusion is applied after label filtering, so only containers that have AutoKuma labels are checked against the patterns.

## AutoKuma-Specific Properties

The following properties are handled internally by AutoKuma and are not passed to Uptime Kuma:

| Property | Example | Description |
|----------|---------|-------------|
| `parent_name` | `apps` | AutoKuma ID of the parent group |
| `notification_name_list` | `["matrix", "discord"]` | AutoKuma IDs of notification providers to enable |
| `tag_names` | `[{"name": "mytag", "value": "val"}]` | Tags to attach to the monitor |
| `docker_host_name` | `local_socket` | AutoKuma ID of the Docker host for a docker monitor |
| `create_paused` | `false` | If `true`, new monitors are added in a paused state |

To reference existing (not AutoKuma-managed) notification providers or tags, use their raw Uptime Kuma IDs instead:

- Notifications: `notification_id_list`, e.g. `kuma.mymonitor.http.notification_id_list: '{"1": true}'`
- Tags: `tags`, e.g. `kuma.mymonitor.http.tags: '[{"tag_id": 2, "value": "optional"}]'`

Both also work in `AUTOKUMA__DEFAULT_SETTINGS`, e.g. `docker.tags: [{"tag_id": 2}]`. A label on the monitor replaces the default for the same setting, it is not merged with it.
