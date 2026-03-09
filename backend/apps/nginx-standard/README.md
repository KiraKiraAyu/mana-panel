# nginx-standard

`nginx-standard` is a **minimal Nginx reverse-proxy template** for Mana Panel.

This package is intentionally simplified (similar to 1Panel app templates):  
advanced routing, SSL certificate lifecycle, WAF, path rules, and policy details should be handled later in the dedicated `Website` feature.

## Overview

- Runtime: Docker Compose
- Default image: `nginx:stable-alpine`
- Category: `proxy`
- Mode: single upstream reverse proxy

## Parameters

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `server_name` | text | ✅ | `_` | Nginx `server_name` value (use `_` as catch-all) |
| `proxy_url` | text | ❌ | (empty) | Optional upstream target URL; empty means static-only hosting |

## Ports

| Key | Container | Default Host Port |
|---|---|---:|
| `http` | `80/tcp` | `80` |

## Rendered files

At install time, Mana Panel renders regular config files:

- `conf/nginx.conf`
- `conf/sites-available/default.conf`
- `conf/includes/http/upgrade_map.conf`
- `conf/includes/http/proxy_base.conf`
- `conf/includes/server/base.conf`

Then it creates links declared by `[[config_link]]`:

- `conf/sites-enabled/default.conf` -> `../sites-available/default.conf`

The template can keep `conf/sites-enabled/` empty.  
Docker Compose mounts both directories into the container.

## Behavior

The generated gateway config keeps `nginx.conf` minimally invasive and delegates reusable policy via `include` files:

- Listens on port `80`
- Matches `server_name`
- Uses Ubuntu-style site layout (`sites-available` + `sites-enabled`)
- `sites-enabled/*.conf` entries are symlinks to `sites-available/*.conf`
- With `proxy_url` set: tries local static files first, then proxies to upstream
- With empty `proxy_url`: serves static files only and returns `404` for missing paths
- Provides `/healthz` endpoint
- Uses shared include snippets for proxy headers, WebSocket upgrade, and basic timeout policy

## Notes

- No HTTPS/TLS setup is included in this template.
- No multi-route/path-routing UI model is included in this template.
- No advanced tuning flags are exposed at install time.

Those capabilities will be added in the upcoming `Website` domain model.
