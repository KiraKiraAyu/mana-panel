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
| `proxy_url` | text | ✅ | `http://host.docker.internal:3000` | Upstream target URL |

## Ports

| Key | Container | Default Host Port |
|---|---|---:|
| `http` | `80/tcp` | `80` |

## Rendered files

At install time, Mana Panel renders and writes:

- `conf/nginx.conf`
- `conf/default/default.conf`

Then Docker Compose mounts them into the container.

## Behavior

The generated site config is intentionally minimal:

- Listens on port `80`
- Matches `server_name`
- Proxies `/` to `proxy_url`
- Adds basic proxy headers (`Host`, `X-Real-IP`, `X-Forwarded-*`)
- Enables WebSocket upgrade headers

## Notes

- No HTTPS/TLS setup is included in this template.
- No multi-route/path-routing UI model is included in this template.
- No advanced tuning flags are exposed at install time.

Those capabilities will be added in the upcoming `Website` domain model.