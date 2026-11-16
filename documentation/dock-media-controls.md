# Dock preview media controls

Click the dock media card to open the media popup. Its transport buttons remain independent click targets.

Dock window previews can control media independently of live/native thumbnails. Open an existing grouped preview, or
right-click any running dock item and choose **Window preview and media controls**, including single-window apps. The
dock's media item and the media popup use the same transport component and Windows command dispatch.

Automatic selection uses an exact, case-insensitive AppUserModelID match, then an executable filename match. It never
selects an unrelated system-default player or guesses from a window title. Windows identifies an app, not a particular
browser tab/window; use the source selector when identities are ambiguous. Previous/play-pause/next/seek availability
comes from Windows and updates with the session. Unsupported/missing sessions return errors. Pending controls block
repeat clicks; an eight-second timeout reports failure without retrying or claiming cancellation. Switching sources
discards feedback from an older source.

## Optional player shortcuts

For NetEase Cloud Music or QQ Music without a Windows session, configure global shortcuts in the player, then enter the
same combinations under **Dock/Taskbar → Media player global shortcuts** and explicitly enable that profile. Profiles
use executable filenames (`cloudmusic.exe` or `qqmusic.exe`) and start disabled with empty bindings. Bindings accept
Ctrl/Alt/Shift plus a letter, digit, F1–F24, arrow, Space, minus or equals. Blank/invalid actions stay disabled.
Combinations must be unique among running apps; no player defaults are assumed.

The backend checks the live window identity, enabled saved profile and existing Run permission. It refuses input while
relevant physical keys are held, and sends one configured chord. Delivery cannot confirm receipt or playback state;
shortcut mode therefore uses a neutral play/pause icon and an explanatory note. There is no automatic broadcast-media
key fallback, process injection, or title matching. Selecting a Windows session still permits explicitly choosing the
configured shortcut fallback instead.

## Checks

```text
npm test
npm run type-check
npm run build:ui
cargo test -p seelen-core --features gen-binds --locked
cargo clippy --locked --all-targets -- -D warnings
```

Regenerate core bindings/package after data/command changes. Automated tests cover matching, shortcut grammar,
backward-compatible opt-in defaults and stale/busy/timeout command behavior; they never send media commands. Manually
test multiple simultaneous players, ambiguous browser sources, unsupported controls, disabled/invalid shortcuts, closing
a source during a pending command, switching sources, and single-window context-menu access. Media buttons must not
activate/minimize the source window.

This feature uses ordinary snapshot previews and does not require DWM thumbnails, full-window peeks or hover leases.
