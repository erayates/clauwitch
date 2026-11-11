# Changelog

## Unreleased

- macOS and Linux support (process detection, launch, data paths, folder opening, theme detection).
- CI builds and checks on Windows, macOS and Linux; release workflow publishes installers for all three.
- Contributor docs: CONTRIBUTING.md, SECURITY.md.
- Fix: sessions of an account that were saved in another profile folder (e.g. from using several accounts in one Claude before Clauwitch) are now copied to that account's folder on switch, so they no longer appear missing.
- Each account shows whether it is signed in.
- Back up an account to a folder and add an account from a backup.
- Clear message, retry button and log entry (`clauwitch.log`) when a folder is locked by another program.
- The window, Alt+F4 and tray Quit are blocked while an account operation is running.
- Claude gets 10 seconds to exit cleanly before it is force-closed; force-closes are logged.

## 1.0.0 — 2026-11-10

- Initial Clauwitch release.
- Switch between Claude Desktop accounts by swapping the active Claude data folder.
- Per-account sessions are kept after a single sign-in.
- Turkish and English UI.
- Follows the active Claude account's light or dark theme.
- Frameless window with in-app minimize, maximize and close controls.
- Custom Clauwitch app icon.
- Frameless rounded window with custom minimize, maximize and close buttons.
- Full-bleed frameless window without the white border; custom edge resizing and reliable maximize.
- Automatic migration from the previous Claude Account Switch data folder.
- Single-instance window, system tray menu, and window position memory.
- Add, rename, switch, close, open folder, and delete account actions.
