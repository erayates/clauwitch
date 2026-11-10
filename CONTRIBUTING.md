# Contributing

Thanks for helping. Bug reports, platform testing (especially macOS and Linux) and pull requests are welcome.

## Setup

See **Build from source** in the [README](README.md). `npm run tauri dev` starts the app with the UI loaded from `ui/` directly; there is no frontend build step.

## Before you open a pull request

```bash
cd src-tauri
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same checks on Windows, macOS and Linux.

## Guidelines

- Keep platform-specific code in `claude_windows.rs` / `claude_unix.rs` or behind `#[cfg(...)]`; shared logic goes in the other modules.
- UI strings live in `ui/main.js`; add both English and Turkish text.
- Clauwitch moves users' Claude session folders. Any change to `switcher.rs` must keep the `state.ini` recovery path working and must never delete data outright (use `paths::move_to_trash`).
- Keep dependencies minimal and avoid network access; the app is offline by design.
- Add a line to `CHANGELOG.md` under **Unreleased**.

## Reporting bugs

Include your OS and version, Claude Desktop version, what you did, and what happened. Do **not** attach your `ClauwitchData` or Claude data folders: they contain session tokens.
