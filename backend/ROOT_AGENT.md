# Root Agent

The backend process requires a reachable root-agent on startup.

## Features

- `systemctl start/stop/restart <unit>`
- `journalctl -u <unit> -n <lines>`
- `kill -KILL/-STOP/-CONT <pid>`
- ensure `/opt/mana-panel` exists and is owned by backend user on startup

All requests require a built-in shared token and go through a strict JSON command whitelist.

## Hardcoded Settings

- Socket path: `/run/mana-panel/root-agent.sock`
- Request timeout: `5000ms`
- Shared token: hardcoded in `backend/src/services/root_agent.rs`

## Run

Panel backend (non-root):

```bash
cargo run
```

Root agent (root):

```bash
sudo -E cargo run -- root-agent
```

## systemd

Reference units are provided in:

- `backend/deploy/systemd/mana-panel-root-agent.service`
- `backend/deploy/systemd/mana-panel-backend.service`
