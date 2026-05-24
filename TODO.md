# TODO

## First Codex pass

1. Run `cargo check` and fix compile issues caused by library API drift.
2. Run `cargo fmt`.
3. Add unit tests for:
   - `21:00 -> 02:00` overnight ranges.
   - `24:00` end normalization.
   - non-recurring next-occurrence logic.
   - overlap calculation with 2, 3, and 5 users.
4. Improve scheduled-session conflict detection by storing participants in a normalized table instead of comma-separated strings.

## MVP after compile

- Test in a private Discord guild.
- Optionally switch command registration from global to guild-only during development.
- Add better Discord embeds.
- Add Discord Scheduled Events creation after `/ss schedule`.
