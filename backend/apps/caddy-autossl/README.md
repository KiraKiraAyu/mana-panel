# caddy-autossl

`caddy-autossl` is a **minimal Caddy reverse-proxy template** for Mana Panel.

This package is intentionally simplified (similar to 1Panel app templates):  
advanced routing policies, per-path rules, upstream pools, middleware chains, and fine-grained security controls should be implemented later in the dedicated `Website` feature.

## Overview

- Runtime: Docker Compose
- Default image: `caddy:2-alpine`
- Category: `proxy`
- Scope: single domain → single upstream reverse proxy

## Parameters

| Key | Type | Required | Default | Description |
|---|---|---:|---|---|
| `email` | text | ❌ | (empty) | Optional email for ACME account registration |

*(Note: Domain routing bindings are now managed dynamically by the Website engine entirely bypassing application form parameters).*

## Ports

| Key | Container | Default Host Port |
|---|---|---:|
| `http` | `80/tcp` | `80` |
| `https` | `443/tcp` | `443` |

## Rendered files

At install time, Mana Panel renders and writes:

- `conf/Caddyfile`

The compose file mounts:

- `./conf/Caddyfile` → `/etc/caddy/Caddyfile` (read-only)
- `./data` → `/data` (cert/storage persistence)
- `./config` → `/config` (runtime config persistence)

## Behavior

The generated Caddy config is intentionally minimal:

- Binds the provided `domain`
- Proxies all traffic to `upstream_host:upstream_port`
- Uses Caddy automatic HTTPS (ACME)
- Outputs logs to stdout

## Requirements

Before deploying in production:

1. Point your domain A/AAAA record to this host.
2. Ensure inbound `80` and `443` are reachable.
3. Ensure upstream host/port is reachable from the container.

## Notes

- No per-path routing model is included.
- No custom Caddy directives are exposed in install form.
- No advanced auth/rate-limit/WAF policy is included.

These will be handled by future `Website` domain features.