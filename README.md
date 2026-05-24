# SessionScheduler

`SessionScheduler` is a Discord bot for organizing tabletop RPG sessions week by week.

The bot exposes one slash command root:

```text
/ss
```

Subcommands:

```text
/ss ican giorno:[lun/mar/merc/gio/ven/sab/dom] start:[HH:MM|HH.MM] end:[HH:MM|HH.MM] ricorrente:[true/false]=false
/ss list
/ss remove id:[xxxxxxxx]
/ss week
/ss overlaps
/ss schedule id:[number]
/ss config channel channel:[#channel]
```

## Core behavior

- Timezone is fixed to `Europe/Rome` for now.
- `/ss week` and `/ss overlaps` work on the next Monday-Sunday week.
- Availability IDs are random 8-character alphanumeric strings.
- `ricorrente` defaults to `false`.
- Non-recurring availability is attached to the next matching weekday/time after the command is run.
- If `end <= start`, the interval crosses midnight.
- `24:00` is accepted only as an end time and is normalized to midnight of the following day.
- Personal responses are ephemeral.
- Normal users can remove only their own availability.
- Admins can remove anyone's availability.
- `/ss overlaps` stores schedulable overlap rows only when run by an admin.
- Any availability change clears stored overlaps, forcing admins to run `/ss overlaps` again.
- `/ss schedule id:[number]` uses the latest valid admin-generated overlap table.
- `/ss schedule` fails with a clear error until `/ss config channel` has been set.
- If a scheduled participant later removes availability that covered a scheduled session, the bot warns the user and DMs the admin who scheduled it.

## Setup

```bash
cp .env.example .env
# edit .env and set DISCORD_TOKEN
cargo run
```

The bot registers commands globally at startup. For development you may want to change registration to guild-only in `src/main.rs` to avoid Discord's global command propagation delay.

## Database

The bot uses SQLite through SQLx. The initial schema lives in:

```text
migrations/0001_init.sql
```

At startup the migration file is applied with `CREATE TABLE IF NOT EXISTS`, so local development is intentionally friction-light.

## Notes for Codex

Read `AGENTS.md` first. It contains project rules and coding expectations.
