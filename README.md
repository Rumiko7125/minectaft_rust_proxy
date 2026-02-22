# Minecraft Rust Proxy

A high-performance Minecraft Java Edition TCP proxy server built with Rust. It mediates connections between Minecraft clients and one or more backend game servers, providing centralized access control, intelligent routing, player authentication, and a full-featured web management interface.

---

## Overview

Minecraft Rust Proxy operates at the TCP layer, accepting inbound Minecraft client connections and forwarding them to the appropriate backend server based on configurable routing rules. All protocol framing, handshake negotiation, and connection lifecycle management are handled natively; the proxy does not depend on any external Minecraft server software.

The proxy is designed for environments where multiple game servers share a single public endpoint — for example, a network of servers exposed through one IP address or a family of subdomains — and where fine-grained control over access, player identity, and observability is required.

Key design goals:

- **Low overhead.** The Rust async runtime (Tokio) handles thousands of concurrent connections with minimal per-connection cost.
- **Operational simplicity.** All configuration is stored in a single SQLite database. There are no configuration files to synchronize across processes.
- **Self-contained deployment.** The web administration interface is compiled into the binary and served directly; no external web server or frontend deployment step is needed in production.

---

## Screenshots

### Login

![init_admin_login.png](screenshot/init_admin_login_1.png)
![init_admin_login_2.png](screenshot/init_admin_login_2.png)

### Initial Setup

![init_admin_1.png](screenshot/init_admin_1.png)
![init_admin_2.png](screenshot/init_admin_2.png)


### Dashboard

![dashboard_1.png](screenshot/dashboard_1.png)
![dashboard_2.png](screenshot/dashboard_2.png)

### Players — Online

![players_1.png](screenshot/players_1.png)
![players_3.png](screenshot/players_3.png)
![players_4.png](screenshot/players_4.png)

### Players — History

![players_2.png](screenshot/players_2.png)

### Access Control

![white_list_1.png](screenshot/white_list_1.png)
![black_list_1.png](screenshot/black_list_1.png)

### Domain Routes

![domain_routes_1.png](screenshot/domain_routes_1.png)
![domain_routes_2.png](screenshot/domain_routes_2.png)
![domain_routes_3.png](screenshot/domain_routes_3.png)
![domain_routes_4.png](screenshot/domain_routes_4.png)

### Backend Server Management

![backend_server_management_1.png](screenshot/backend_server_management_1.png)
![backend_server_management_2.png](screenshot/backend_server_management_2.png)
![backend_server_management_3.png](screenshot/backend_server_management_3.png)
![backend_server_management_4.png](screenshot/backend_server_management_4.png)
![backend_server_management_5.png](screenshot/backend_server_management_5.png)
![game_1.png](screenshot/game_1.png)

### Two-Factor Authentication

![two_factor_1.png](screenshot/two_factor_1.png)

### Logs — Moderation

![logs_1.png](screenshot/logs_1.png)

### Logs — Sessions

![logs_2.png](screenshot/logs_2.png)

### Config

![config.png](screenshot/config.png)

### Account

![account_1.png](screenshot/account_1.png)

### In-Game Limbo — 2FA Enrollment

![limbo_1.png](screenshot/limbo_1.png)
![limbo_2.png](screenshot/limbo_2.png)
![limbo_3.png](screenshot/limbo_3.png)
![limbo_4.png](screenshot/limbo_4.png)

---

## Features

### Protocol Support

The proxy handles connections from clients running the following Minecraft Java Edition versions. Clients using any other version receive an informative disconnect message.

| Version Range | Protocol Number |
|---|---|
| 1.7.2 – 1.7.10 | 4 – 5 |
| 1.8 – 1.8.9 | 47 |
| 1.21 – 1.21.1 | 767 |
| 1.21.2 – 1.21.3 | 768 |
| 1.21.4 | 769 |
| 1.21.5 | 770 |
| 1.21.6 | 771 |
| 1.21.7 – 1.21.8 | 772 |
| 1.21.9 – 1.21.10 | 773 |
| 1.21.11+ | 774+ |

### Multi-Backend Routing

Multiple backend game servers can be registered. The proxy selects the destination for each incoming connection using the following resolution order:

1. **Domain route** — a regex pattern matched against the hostname the client connected with (the `server_address` field in the Minecraft handshake). Each domain route references a backend server and carries a configurable priority.
2. **Default backend** — the backend marked as the system default, used when no domain route matches.

If the resolved backend is disabled or in maintenance mode, the client receives an appropriate status message and the connection is refused cleanly.

### MOTD and Status Handling

Each backend server carries an optional custom MOTD JSON string. When a client queries the server list, the proxy constructs a response using the backend's MOTD, injecting the connecting client's protocol version number so that the displayed version text matches. Two additional modes are available per backend:

- **MOTD passthrough** — the proxy opens a temporary connection to the backend and returns the backend's live MOTD and favicon verbatim.
- **Ping passthrough** — the proxy uses the backend's live latency measurement in the status response.

### Access Control

- **Whitelist** — when enabled, only players on the whitelist may connect. The whitelist can be toggled without restarting the proxy.
- **Blacklist** — players on the blacklist are refused at the login stage with a configurable reason message. Ban, pardon, and kick operations are recorded to a moderation log.

### Player Two-Factor Authentication (2FA)

When 2FA is enabled globally, new players are held in a Limbo session after login. During the Limbo session:

- A QR code is rendered as a Minecraft map item placed in the player's hotbar, allowing the player to scan it with any TOTP-compatible authenticator application directly from the in-game map view.
- A clickable web link to an externally hosted QR image is also provided via chat for clients that cannot view maps (e.g., 1.7.x).
- The player enters their six-digit TOTP code in chat. Once verified, they are disconnected and prompted to reconnect, at which point the proxy forwards the connection normally.
- Verified sessions are cached for a configurable number of hours so that returning players are not re-prompted on every login.

Administrators can view bound players and revoke 2FA secrets from the web interface or CLI.

### Trusted Domain Enforcement

An optional trusted domain can be configured. When set:

- Clients connecting by direct IP address are permitted but routed only to the default backend.
- Clients connecting with a hostname that does not match the trusted domain (or one of its subdomains) receive a rejection message during the Status phase and the Login phase.

This allows the proxy to expose different behavior under a canonical domain while still allowing operators to connect directly by IP for diagnostics.

### Web Administration Interface

A browser-based management interface is served by the proxy process itself on a configurable address and port (default `127.0.0.1:20220`). The interface provides:

- **Dashboard** — online player count, aggregate traffic statistics, recent session activity, and proxy uptime with automatic refresh.
- **Players** — live list of connected players with kick action; historical session log with CSV export.
- **Access Control** — whitelist and blacklist management with add, remove, and toggle operations.
- **Routes** — domain route management with regex pattern editor, backend association, priority, and MOTD preview; JSON export and import.
- **Backend** — full lifecycle management of backend servers including creation, editing, enable/disable, maintenance mode toggle, and deletion.
- **Two-Factor** — per-player 2FA binding list with QR code viewer and revoke action.
- **Logs** — paginated moderation log and session log; individual CSV export for each.
- **Config** — system settings editor; full data migration with ZIP archive export (JSON tables plus CSV logs) and import.
- **Account** — password change and TOTP-based 2FA for the administrator account itself.

The interface uses JWT bearer token authentication with an eight-hour session lifetime. An initial administrator account is created on first launch through a guided setup screen.

### Runtime CLI

An interactive command-line interface is available while the proxy is running. It exposes the same operations as the web interface for environments where browser access is not available or is undesirable.

```
  list players                       List online players
  list whitelist                     List whitelist
  list blacklist                     List blacklist
  kick <name> [reason]               Kick a player
  ban <name> [reason]                Ban a player
  pardon <name>                      Unban a player
  whitelist on|off                   Enable/disable whitelist
  whitelist add|remove <name>        Modify whitelist
  backend list                       List all backend servers
  backend add <name> <addr> <port>   Add backend server
  backend enable|disable <id>        Toggle backend status
  backend maintenance <id> on|off    Toggle maintenance mode
  2fa list                           List 2FA-enabled players
  2fa remove <name>                  Revoke player 2FA
  config list                        Show current configuration
  config set <key> <value>           Update configuration key
  config reload                      Reload configuration from database
  modlog [player] [n]                Show moderation log
```

### Internationalization

Player-facing messages (login rejection, 2FA prompts, server status messages) are loaded from JSON language files at startup. The following languages are supported:

- English (`en`)
- Simplified Chinese (`zh-CN`)
- Traditional Chinese (`zh-TW`)
- Japanese (`ja`)
- Russian (`ru`)
- German (`de`)
- French (`fr`)
- Korean (`ko`)

The language can be configured globally and overridden per backend server. The web administration interface independently supports all of the above locales, switchable at any time from the login page or the sidebar.

### Session and Audit Logging

Every player connection is recorded to the `player_sessions` table with login and logout timestamps, upload and download byte counts, protocol version, and the kick reason if applicable. Moderation actions (kick, ban, pardon) are recorded to a separate `moderation_logs` table with operator identity and timestamp. Both logs are queryable and exportable from the web interface.

---

## Requirements

### Runtime

- Linux, macOS, or Windows (64-bit)
- SQLite 3 (embedded via `sqlx`; no separate installation required)

### Build

- Rust 1.75 or later (edition 2021)
- `cargo`
- Node.js 18 or later and `npm` (required only to rebuild the frontend; the repository includes a pre-built `web/dist`)

---

## Building from Source

The repository contains a pre-built frontend (`web/dist`) that is embedded directly into the binary at compile time, so a Node.js installation is not required for a standard backend-only build. Rebuild the frontend only if you have modified the Vue source.

### Linux

#### Debian / Ubuntu

```bash
# Install system dependencies
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# (Optional) Install Node.js for frontend rebuild
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Clone and build
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy

# (Optional) Rebuild the frontend
cd web && npm install && npm run build && cd ..

# Build the release binary
cargo build --release
```

The compiled binary is at `target/release/minecraft-rust-proxy`.

#### RHEL / Fedora / Rocky Linux

```bash
# Install system dependencies
sudo dnf install -y curl gcc openssl-devel pkg-config

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# (Optional) Install Node.js
sudo dnf install -y nodejs npm

# Clone and build
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy
cargo build --release
```

#### Generic Linux

Any distribution with a C linker, OpenSSL development headers, and pkg-config installed is sufficient. Install Rust through [rustup.rs](https://rustup.rs) and run `cargo build --release` from the repository root.

---

### macOS

#### Intel (x86_64)

```bash
# Install Xcode Command Line Tools (provides clang and linker)
xcode-select --install

# Install Homebrew if not already present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# (Optional) Install Node.js
brew install node

# Clone and build
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy
cargo build --release
```

#### Apple Silicon (M1 / M2 / M3)

The steps are identical to Intel. Rustup detects the `aarch64-apple-darwin` host automatically and installs the correct toolchain. The resulting binary runs natively on Apple Silicon with no Rosetta translation needed.

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy
cargo build --release
```

The compiled binary is at `target/release/minecraft-rust-proxy`.

---

### Windows

#### Using winget (Windows 10 1709+ / Windows 11)

```powershell
# Install Rust (includes cargo)
winget install Rustlang.Rustup

# Install Visual Studio Build Tools (C++ linker required by Rust on Windows)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# (Optional) Install Node.js for frontend rebuild
winget install OpenJS.NodeJS

# Restart your terminal, then clone and build
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy
cargo build --release
```

#### Using Scoop

```powershell
# Install Scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression

# Install Rust and Node.js
scoop install rustup nodejs

# Initialize rustup
rustup-init.exe -y
# Restart your terminal

# Clone and build
git clone https://github.com/your-username/minecraft-rust-proxy.git
cd minecraft-rust-proxy
cargo build --release
```

#### Manual Installation

1. Download and run the Rust installer from [rustup.rs](https://rustup.rs). Select the default installation, which installs `rustup`, `rustc`, and `cargo`.
2. Install [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the **Desktop development with C++** workload. This provides the MSVC linker required by the Rust Windows target.
3. (Optional) Download Node.js 20 LTS from [nodejs.org](https://nodejs.org) if you intend to rebuild the frontend.
4. Open a new Command Prompt or PowerShell window, navigate to the cloned repository, and run:

```powershell
cargo build --release
```

The compiled binary is at `target\release\minecraft-rust-proxy.exe`.

#### Notes on Windows

- The MSVC toolchain (`stable-x86_64-pc-windows-msvc`) is the recommended and default target on Windows. The GNU toolchain (`x86_64-pc-windows-gnu`) via MSYS2 also works but requires additional setup.
- Windows Defender may scan the binary on first execution, causing a brief delay. This is normal behavior.
- The proxy binds to `0.0.0.0:25565` by default. Windows Firewall will prompt to allow the connection on first launch; allow access for the network types you require.

---

### Rebuilding the Frontend

The pre-built frontend in `web/dist` is embedded into the binary at compile time via `rust-embed`. To rebuild it after modifying the Vue source files:

```bash
cd web
npm install
npm run build
cd ..
cargo build --release
```

The `cargo build` step must follow `npm run build` so that the updated `web/dist` contents are picked up by the embed macro.

---

## Running

On first launch, the proxy creates `proxy.db` in the working directory and runs all database migrations automatically.

```bash
# Linux / macOS — start with default settings
# (binds Minecraft listener to 0.0.0.0:25565, web interface to 127.0.0.1:20220)
./target/release/minecraft-rust-proxy

# Windows
.\target\release\minecraft-rust-proxy.exe
```

After startup, open the web administration interface in a browser:

```
http://127.0.0.1:20220
```

The first visit presents a setup screen to create the initial administrator account. Once completed, the dashboard is available and backend servers and routing rules can be configured.

---

## Configuration Reference

All configuration is managed through the web interface or the `config set` CLI command. Settings are persisted in the `settings` table of `proxy.db`.

| Key | Default | Description |
|---|---|---|
| `local_address` | `0.0.0.0` | Address the proxy listens on for Minecraft connections |
| `local_port` | `25565` | Port the proxy listens on for Minecraft connections |
| `web_api_enable` | `true` | Enable the web administration interface |
| `web_api_address` | `127.0.0.1` | Bind address for the web interface |
| `web_api_port` | `20220` | Port for the web interface |
| `whitelist_enabled` | `false` | Enforce the player whitelist |
| `enable_2fa` | `false` | Require TOTP 2FA for all connecting players |
| `two_factor_session_hours` | `12` | Duration in hours before a verified 2FA session expires |
| `two_factor_issuer` | `MinecraftProxy` | Issuer name shown in the authenticator app |
| `language` | `en` | Language for player-facing messages |
| `trusted_domain` | _(empty)_ | Canonical hostname; if set, connections from other hostnames are rejected |
| `log_dir` | `./logs` | Directory for connection log files |
| `show_log_level` | `0` | Console log verbosity (0 = minimal) |
| `save_log_level` | `0` | File log verbosity (0 = minimal) |

---

## Data Migration

The web interface provides full data export and import under the Config page. The export produces a ZIP archive containing:

- `manifest.json` — export metadata and schema version
- `settings.json` — current configuration values
- `backends.json` — backend server definitions
- `domain_routes.json` — domain routing rules
- `whitelist.json` — whitelist entries
- `blacklist.json` — blacklist entries
- `two_factor_secrets.json` — player 2FA bindings
- `moderation_logs.csv` — complete moderation history
- `player_sessions.csv` — complete session history

The import endpoint accepts a ZIP archive in the same format and merges the data into the running database without interrupting active connections.

---

## REST API

The web interface communicates with the proxy through a versioned REST API available at `/api/v1/`. All endpoints except the authentication group require a `Bearer` token obtained at login.

Authentication endpoints (no token required): `POST /api/v1/auth/login`, `POST /api/v1/auth/setup`, TOTP setup and verification.

Protected endpoint groups:

| Prefix | Description |
|---|---|
| `/api/v1/dashboard` | Summary statistics and recent activity |
| `/api/v1/players` | Online players, session history, kick |
| `/api/v1/whitelist` | Whitelist management |
| `/api/v1/blacklist` | Blacklist management |
| `/api/v1/routes/domain` | Domain routing rules |
| `/api/v1/backend` | Backend server management |
| `/api/v1/2fa/players` | Player 2FA management |
| `/api/v1/config` | System configuration read/write |
| `/api/v1/logs` | Moderation and session logs with export |
| `/api/v1/admin` | Administrator account management |
| `/api/v1/migration` | Data export and import |
| `/api/v1/ping` | Live Minecraft status ping to a target server |

---

## Security and Privacy

Minecraft Rust Proxy operates exclusively at the TCP transport layer. Its role is to accept an inbound TCP connection, determine the appropriate backend server, and bidirectionally relay the raw byte stream between the client and that backend. The proxy does not decrypt, modify, store, or analyse the content of Minecraft game packets beyond what is minimally required to complete the initial handshake (reading the `server_address` field to apply domain routing rules) and the login sequence (reading the player username to apply access control and session logging).

Specifically:

- **No packet snooping.** Once the connection is forwarded to a backend, all subsequent traffic is relayed as an opaque byte stream. The proxy does not parse Play-phase packets.
- **No credential interception.** The proxy does not perform any form of man-in-the-middle decryption. Mojang's online-mode encryption is negotiated end-to-end between the Minecraft client and the backend server; the proxy passes the encrypted bytes through unchanged.
- **No data exfiltration.** Session metadata (username, UUID, connection timestamps, byte counters) is written only to the local SQLite database on the host where the proxy runs. No data is transmitted to any third party.
- **Minimal surface area.** The web administration interface binds to `127.0.0.1` by default and is not reachable from the public network without explicit reconfiguration.

The source code is fully open and auditable. Users are encouraged to review the `src/proxy/` directory to verify these claims independently.

---

## License

```
MIT License

Copyright (c) 2026 7125Jk

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
