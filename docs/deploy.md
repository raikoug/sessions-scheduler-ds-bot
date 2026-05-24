# Deploy Guide

This deployment flow follows the same segregation model used in the reference bot:

- dedicated service user
- application files under `/opt/session_scheduler`
- runtime data under the service user home
- `systemd` service with basic hardening

The goal is to keep code, secrets, and SQLite runtime data separated.

## Recommended layout

Example layout:

```text
/opt/session_scheduler/
├── .env
└── session_scheduler

/home/session_scheduler/
└── .session_scheduler/
    └── session_scheduler.sqlite
```

## 1. Create service user and directories

Create a dedicated service user, the application directory, and the runtime data directory:

```bash
sudo useradd --system --create-home --home-dir /home/session_scheduler session_scheduler
sudo mkdir -p /opt/session_scheduler
sudo mkdir -p /home/session_scheduler/.session_scheduler
sudo chown -R session_scheduler:session_scheduler /opt/session_scheduler /home/session_scheduler
```

## 2. Build and install the binary

Build the project in release mode:

```bash
cargo build --release
```

Copy the binary and environment template:

```bash
sudo cp target/release/session_scheduler /opt/session_scheduler/
sudo cp .env.example /opt/session_scheduler/.env
sudo editor /opt/session_scheduler/.env
sudo chown -R session_scheduler:session_scheduler /opt/session_scheduler /home/session_scheduler
sudo chmod 755 /opt/session_scheduler /home/session_scheduler /home/session_scheduler/.session_scheduler
```

## 3. Configure the environment

In `/opt/session_scheduler/.env`, set at least:

```dotenv
DISCORD_TOKEN=your-discord-bot-token
DATABASE_URL=sqlite:///home/session_scheduler/.session_scheduler/session_scheduler.sqlite
RUST_LOG=session_scheduler=info,poise=info,serenity=info
```

Notes:

- do not commit `.env`
- prefer an absolute SQLite path in production
- the `session_scheduler` user must be able to write `/home/session_scheduler/.session_scheduler/`

## 4. Create the systemd unit

Create `/etc/systemd/system/session_scheduler.service`:

```ini
[Unit]
Description=SessionScheduler Discord bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=session_scheduler
Group=session_scheduler
WorkingDirectory=/opt/session_scheduler
EnvironmentFile=/opt/session_scheduler/.env
ExecStart=/opt/session_scheduler/session_scheduler
Restart=always
RestartSec=5
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=false
ReadWritePaths=/home/session_scheduler/.session_scheduler /opt/session_scheduler

[Install]
WantedBy=multi-user.target
```

Then reload and enable the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now session_scheduler
sudo systemctl status session_scheduler
```

## 5. Logs

Follow logs with:

```bash
journalctl -u session_scheduler -f
```

## 6. Update flow

Typical update flow:

```bash
git pull
cargo build --release
sudo cp target/release/session_scheduler /opt/session_scheduler/session_scheduler
sudo chown session_scheduler:session_scheduler /opt/session_scheduler/session_scheduler
sudo systemctl restart session_scheduler
sudo systemctl status session_scheduler
```

## 7. First validation

Before installing the service, validate locally:

```bash
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run
```

If the bot connects correctly, stop it and continue with the service setup.

## 8. Backup

The critical runtime file is:

```text
/home/session_scheduler/.session_scheduler/session_scheduler.sqlite
```

You may also want to back up:

```text
/opt/session_scheduler/.env
```

## 9. Troubleshooting

If the service does not start, verify first that these paths exist:

- `/opt/session_scheduler/session_scheduler`
- `/opt/session_scheduler/.env`
- `/home/session_scheduler/.session_scheduler`

If the bot starts but does not persist data:

- check `DATABASE_URL`
- confirm the SQLite path is writable by `session_scheduler`
- inspect logs with `journalctl -u session_scheduler -f`

If the bot starts but does not connect:

- check `DISCORD_TOKEN`
- check outbound network access to Discord
- inspect logs with `journalctl -u session_scheduler -f`
