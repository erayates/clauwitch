# Manual test checklist

Run before a release. CI covers build, clippy and unit tests; these need a real machine with Claude Desktop.

**Back up your Claude data folder first.**

## Environments

| Area | Cases |
|---|---|
| OS | Windows 10, Windows 11, macOS (Apple Silicon), Ubuntu with a community `claude-desktop` |
| Display | 100%, 125%, 150%, 200% scaling; two monitors; unplug the monitor the window was on, then use **Settings → Reset window position** |
| Accounts | 1, 2 and 10+ profiles (search box appears above 5) |

## Scenarios

- [ ] Add an account, sign in, switch back and forth: each account keeps its sign-in and sessions.
- [ ] Switch while Claude is open: confirmation appears; Claude closes and reopens on the target account.
- [ ] Switch while Claude is closed: no confirmation, Claude starts on the target account.
- [ ] Account that used to share a folder with another account: its sessions appear after one switch.
- [ ] Lock the profile folder (open a terminal inside it): switch shows the "folder in use" message with **Try again**; `clauwitch.log` has an entry.
- [ ] Close the window / Alt+F4 / tray **Quit** during a switch: blocked.
- [ ] Kill Clauwitch during a switch (Task Manager): next start restores a consistent state.
- [ ] Back up an account, delete it, add it from the backup: sign-in and sessions are there.
- [ ] Settings: theme Automatic/Light/Dark; close to tray on/off; start with system on → reboot → app is in the tray, no window.
- [ ] Tray: active account shown; **Open Claude**, **Settings**, account switch, left-click opens the window.
- [ ] Settings → **Save diagnostics**: file contains no account names, tokens or user name.
- [ ] Claude Desktop not installed: clear message, no crash.
- [ ] Corrupt `settings.ini` (random text): app starts with the original account.
- [ ] Offline: everything works (the app makes no network requests).
- [ ] Keyboard only: Tab through header, list (arrow keys), menus (arrow keys, Esc), dialogs (focus stays inside).
- [ ] Windows High Contrast on: borders, selection and focus are visible.
- [ ] Turkish and English UI texts fit and read correctly.
