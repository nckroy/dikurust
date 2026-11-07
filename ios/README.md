# Diku Wizard

An iPhone and iPad app for keeping an eye on diku.earth and running the game
as an immortal.

| Tab | What it does | Needs |
|-----|--------------|-------|
| **Status** | Up or down, uptime, people online, wizlock, game time and weather | Nothing: it reads the public `/status` |
| **Players** | Everyone connected, link-dead or still logging in: level, class, room, address, idle time, HP/mana/move, flags. Per player: `stat`, `where`, `goto`, `transfer`, `restore`, `tell`, `force`, `snoop`, `noshout`/`notell`/`noemote`/`freeze`, `advance`, `purge` | Level 21+ (each action shows only when your level allows it) |
| **World** | `look`, `stat room`, `goto`, `at … look`, `where`, `load char/obj`, `purge`, `wiz`, `echo`, `shout`; every zone with its reset countdown and who's in it | Level 21+ |
| **Console** | The game itself, with quick commands and history | Any character |
| **Wizard → Server** | Save, `transfer all`, wizlock, reboot, shutdown | Level 24 for the last three |
| **Wizard → Server Log** | The last 300 lines of the server log, searchable | Level 24 |

On iPad the sections are in a sidebar.

## How it talks to the server

The app logs in over `/ws` exactly as the web client does, with your
character's name and password. The password is kept in the device's
Keychain. Every wizard action is an ordinary game command, so the game's own
level checks and logging apply, as if you had typed it.

For the structured views (Players, World, Log) the app sends an out-of-band
frame, `\x01snap`, on that logged-in socket. The server answers immortals
with a JSON snapshot and refuses everyone else. See `src/admin.rs`.

Things to know:

- **One login at a time.** The game refuses a second login for a character
  already playing ("Already playing, cannot connect"). Quit the web client
  first, or let its link drop.
- Closing the app or losing the network drops the link. Your character stays
  in the game, link-dead, and the app reconnects into it next time. Use
  **Quit the game** in Console or Settings to leave cleanly.
- **Wizlock locks the app out too.** Lift it before you disconnect.
- On diku.earth, `shutdown` and `shutdown reboot` both bring the world back
  in a few seconds, because the container restarts itself.

## Building

Open `ios/DikuWizard.xcodeproj` in Xcode, choose your team under *Signing &
Capabilities*, and run on your phone or iPad. It targets iOS 17 and has no
dependencies.

The server needs the build that has `/status` (`src/admin.rs`). Against an
older server the Status tab says there's no `/status` endpoint, and the
wizard views wait forever for a snapshot.

To try it against a local server, run `cargo run --release` and set the
server to `http://127.0.0.1:4000` in Settings. The app allows plain HTTP
only for local network addresses.
