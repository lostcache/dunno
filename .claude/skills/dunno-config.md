# Dunno Configuration Skill

## Overview

Dunno is a Rust CLI that captures coding knowledge using a graph-based hierarchy. This skill documents how configuration works in dunno.

## Configuration Priority (Per-Field)

Configuration is resolved on a **per-field basis** from highest to lowest priority:

1. **CLI flags** (`--backend`) - highest priority, overrides everything
2. **Local project config** (`./dunno.toml`) - project-specific settings
3. **Global user config** (`~/.config/dunno/dunno.toml`) - user-wide settings  
4. **Environment variables** - system-level overrides
5. **Built-in defaults** - fallback values

Each field is resolved independently. If a field is missing from a higher-priority source, the next source is checked.

## Config File Locations

```
./dunno.toml              # Local project config (higher priority)
~/.config/dunno/dunno.toml # Global user config (lower priority)
```

## Configuration Fields

### Backend Selection
- `backend`: `"local"` | `"cloud"`
- Local: Uses embedded SurrealDB
- Cloud: Connects to SurrealDB cloud instance

### Local Backend Settings
```toml
[local]
path = "~/.local/share/dunno/data.db"  # Database file path
```

### Cloud Backend Settings
```toml
[cloud]
url = "wss://YOUR_INSTANCE.surrealdb.com"
namespace = ""
database = ""
username = "root"
password = "root"
auth_type = "root"  # "root" | "namespace" | "database"
```

### Additional Settings
```toml
qdrant_url = "mem://"  # For vector search (optional)
```

## Environment Variables

All configuration fields can be set via environment variables:

- `DUNNO_BACKEND` - Backend type
- `DUNNO_LOCAL_PATH` - Local database path
- `DUNNO_CLOUD_URL` - Cloud URL
- `DUNNO_CLOUD_NS` - Namespace
- `DUNNO_CLOUD_DB` - Database name
- `DUNNO_CLOUD_USER` - Username
- `DUNNO_CLOUD_PASS` - Password
- `DUNNO_CLOUD_AUTH_TYPE` - Authentication type

## Priority Example

**Global config** (`~/.config/dunno/dunno.toml`):
```toml
backend = "cloud"
[cloud]
url = "wss://global.example.com"
namespace = "global-ns"
database = "global-db"
username = "global-user"
password = "global-pass"
```

**Local config** (`./dunno.toml`):
```toml
[cloud]
url = "wss://local.example.com"
# namespace not specified
```

**Effective configuration**:
- `backend` = `"cloud"` (from global - not in local)
- `cloud.url` = `"wss://local.example.com"` (from local - overrides global)
- `cloud.namespace` = `"global-ns"` (from global - not in local)
- `cloud.database` = `"global-db"` (from global - not in local)
- `cloud.username` = `"global-user"` (from global - not in local)
- `cloud.password` = `"global-pass"` (from global - not in local)

## Viewing Current Configuration

```bash
dunno config show
```

This displays the resolved configuration with both config file paths and sensitive data redacted.

## CLI Override

Override backend for a single command:
```bash
dunno --backend cloud config show
```

## Default Values

- **backend**: `local`
- **local.path**: `~/.local/share/dunno/data.db`
- **cloud.username**: `root`
- **cloud.password**: `root`
- **cloud.auth_type**: `root`
- **qdrant_url**: `mem://`

## Common Patterns

### Project-specific database path
Create `./dunno.toml`:
```toml
[local]
path = "./.dunno/data.db"
```

### Using cloud in one project only
Create `./dunno.toml`:
```toml
backend = "cloud"
[cloud]
url = "wss://my-instance.surrealdb.com"
namespace = "my-project"
database = "dunno"
```

### Environment-specific settings
Use environment variables in CI/CD:
```bash
export DUNNO_BACKEND=cloud
export DUNNO_CLOUD_URL="wss://prod.surrealdb.com"
export DUNNO_CLOUD_PASS="$SECRET_PASSWORD"
dunno config show
```
