# Deploy Guide

This guide describes a simple production deployment for `SessionScheduler` on a Linux host.

## Prerequisites

- Rust toolchain installed
- A Discord bot token with the slash commands configured in the Developer Portal
- A writable directory for the SQLite database
- A Linux service manager such as `systemd`

## Environment

Create a local environment file from the example:

```bash
cp .env.example .env
```

Set at least:

```dotenv
DISCORD_TOKEN=your-discord-bot-token
DATABASE_URL=sqlite://session_scheduler.sqlite
RUST_LOG=session_scheduler=info,poise=info,serenity=info
```

Notes:

- Do not commit `.env`
- `DATABASE_URL` is relative to the current working directory unless you use an absolute path
- Make sure the process user can write the SQLite file

## Build

For a release build:

```bash
cargo build --release
```

The production binary will be created at:

```text
target/release/session_scheduler
```

## First local validation

Before installing the service, validate the configuration:

```bash
cargo check
cargo test
cargo run
```

If the bot starts correctly, stop it and continue with the service setup.

## Run directly

You can run the release binary directly:

```bash
set -a
source .env
set +a
./target/release/session_scheduler
```

This is useful for smoke testing before enabling the service.

## Suggested directory layout

Example:

```text
/opt/session_scheduler/
├── .env
├── session_scheduler.sqlite
└── session_scheduler
```

Example install commands:

```bash
mkdir -p /opt/session_scheduler
cp target/release/session_scheduler /opt/session_scheduler/session_scheduler
cp .env /opt/session_scheduler/.env
```

## systemd service

Example unit file:

```ini
[Unit]
Description=SessionScheduler Discord bot
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/session_scheduler
EnvironmentFile=/opt/session_scheduler/.env
ExecStart=/opt/session_scheduler/session_scheduler
Restart=always
RestartSec=5
User=session_scheduler
Group=session_scheduler

[Install]
WantedBy=multi-user.target
```

Save it as:

```text
/etc/systemd/system/session_scheduler.service
```

Then reload and enable the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now session_scheduler
sudo systemctl status session_scheduler
```

## Logs

Read service logs with:

```bash
journalctl -u session_scheduler -f
```

## Updating the bot

Typical update flow:

```bash
git pull
cargo build --release
cp target/release/session_scheduler /opt/session_scheduler/session_scheduler
sudo systemctl restart session_scheduler
sudo systemctl status session_scheduler
```

## Backup

The critical runtime state is the SQLite database file.

At minimum, back up:

```text
/opt/session_scheduler/session_scheduler.sqlite
```

If needed, also back up:

```text
/opt/session_scheduler/.env
```

## Common problems

### Bot starts but does not connect

- Check `DISCORD_TOKEN`
- Check outbound network access to Discord
- Check logs with `journalctl -u session_scheduler -f`

### Bot starts but does not persist data

- Check that the process user can write the SQLite file
- Check `DATABASE_URL`
- Confirm the working directory matches the database path assumptions

### Slash command changes do not appear immediately

- The bot currently registers commands globally at startup
- Global Discord command propagation can take time

