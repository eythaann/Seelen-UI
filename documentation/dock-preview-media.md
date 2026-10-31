# Media controls in dock previews

Hover over a running app's dock item for 350 ms to open its thumbnail preview. The pointer can cross the gap into the
popup without dismissing it. Hovering does not request keyboard focus; clicking inside the popup does. Clicking the
thumbnail activates that existing window handle. Playback buttons are separate controls and do not activate the app.

## Windows media sessions

The preview subscribes to the existing media-session events. Automatic selection first uses an exact, case-insensitive
AppUserModelID match, then an executable filename match. It never defaults to an unrelated system-recommended player.
When Windows does not expose an identity that can be matched, the source selector lets the user choose explicitly.
Windows media sessions identify a source app; they do not reliably identify an individual browser window or tab.

Previous, play/pause, next and seek availability come from the player's Windows playback controls and are updated on
playback-info changes. Both the dock media item and media popup use the same transport component as the preview. Missing
sessions, unsupported actions and rejected Windows commands return errors. The component prevents repeated clicks while
a command is pending, reports failure or an eight-second timeout, and ignores old-source replies after a source switch.
A timeout is not a cancellation of the Windows operation, and no command is automatically retried.

## NetEase Cloud Music and QQ Music fallback

In **Dock/Taskbar → Media player global shortcuts**, enter the global shortcuts already configured and enabled in the
player, then explicitly enable that player's fallback. Profiles start disabled with empty bindings. They are stored
under `byWidget["@seelen/weg"].mediaShortcuts`, keyed by executable filename. For example, only if these combinations
have also been configured in the player:

```json
{
  "cloudmusic.exe": {
    "enabled": true,
    "previous": "Ctrl+-",
    "togglePlayPause": "Ctrl+0",
    "next": "Ctrl+="
  }
}
```

QQ Music has its own `qqmusic.exe` profile. Its combinations must be chosen by the user; no player-specific defaults are
assumed. If a Windows session is also available, the source selector allows explicitly choosing the configured shortcut
fallback instead. The configured combinations must be unique across running apps.

The backend verifies the live window's executable name and the saved, enabled profile. It accepts one modified
virtual-key chord, checks for physically held keys, and sends only that chord. It does not use broadcast media keys, UI
Automation searches or process injection. Third-party widgets require the existing `Run` permission for this command.
Successful input delivery cannot confirm receipt by the player or report playback state. Accordingly, shortcut mode
displays a neutral play/pause glyph and an explanation rather than claiming a known playing state. Blank or invalid
bindings disable their respective buttons.

## Validation

```text
npm test
npm run type-check
npm run build:ui
cargo test -p seelen-core --features gen-binds --locked
cargo check -p seelen-ui --bin seelen-ui --locked
```

Regenerate the core bindings/package after changing the Rust command or data declarations. Only English entries are
added while iterating; the repository's translation workflow owns other locales.

Manual desktop acceptance is still required:

- Type in another app, hover a dock item, and confirm the preview does not steal keyboard focus. Cross into the popup,
  operate its controls, then leave. Repeat on each dock side and on monitors with different scale factors.
- Play browser media alongside NetEase Cloud Music or QQ Music. Confirm automatic selection stays with the hovered app,
  and explicitly select a browser source when the Windows identity is ambiguous.
- Configure unique global shortcuts in each player, enable its profile, and test each action. Confirm disabled, missing
  and invalid profiles do not send input. Check that media buttons do not open or minimize the player window.
- Close/restart a source while the popup is open. Confirm stale commands report failure; switch sources during a slow
  command and confirm its reply does not change the newly selected source's feedback.

## Temporary full-window preview

The thumbnail image uses a persistent DWM thumbnail relationship with the real source window. Video frames are composed
by Windows; they are not captured/encoded as WebP or transported as base64 to the webview. The frontend sends physical
layout/opacity changes during popup animation and a lightweight 80 ms lease/pointer heartbeat when layout is stable.
Only one layout request can be in flight, and intermediate animation layouts are coalesced into the latest frame.

When DWM cannot provide a surface, only the visible fallback windows request snapshots, at most four times per second.
Capture requests are deduplicated while queued or running. The popup does not fetch the large global snapshot cache on
the live path. Missing frames show a descriptive placeholder rather than a question mark. Minimized or non-rendering
source windows may expose only their last composed frame, just as with other DWM thumbnails. No measured frame-rate
guarantee is made; protected-content handling and real video smoothness require desktop acceptance.

Native relationships are scoped to their requesting webview/session, unregistered on cancellation, destination hide,
lease expiry and application exit. Cancelled late requests cannot resurrect an old surface. English translation
fallbacks no longer log errors for every control rerender, avoiding repeated UI-to-backend log traffic during playback.

The four tests that register actual DWM surfaces are ignored in ordinary headless test runs. Run them on an interactive
Windows desktop with:

```text
cargo test -p seelen-ui live_previews::tests -- --include-ignored --test-threads=1
```

Large groups are scrollable, and only thumbnails intersecting the visible list consume the 16 live slots; the hovered
thumbnail takes priority. The backend bounds registrations without rejecting an oversized group, so the popup still
receives visibility/pointer state. If a layout request is rejected, a target-free recovery request retrieves that state
and enables snapshot fallback; retries back off instead of generating per-frame failures. Only visible requested
thumbnails trigger snapshot fallback.

Temporary-window restoration keeps multiple PID-checked neighbors above and below the source. If an upper neighbor
closes or changes its topmost band, another survivor restores the old position. A lower survivor can also anchor the
restore. If all saved anchors vanish, the source is placed behind surviving visible windows in the same band without
moving it behind the desktop or clearing its topmost flag.

Hover any running application's thumbnail for 450 ms to temporarily present that exact window without changing keyboard
focus. The pointer may move from the thumbnail onto the real window. Leaving both regions restores its previous z-order
and minimized state after a short grace period. Clicking the thumbnail or the presented window keeps that same HWND;
there is no hide/reopen handoff. Media controls are outside the full-window hover region.

The backend owns a single preview lease, scoped to the requesting webview and a unique session token. It checks the
tracked app window and `ManageAppWindows` permission, rejects cancelled late begins, and restores temporary state when
the popup disappears, heartbeats stop, or Seelen exits. A foreground change to a different application cancels the
preview. User activation of the presented window commits it. No browser-specific APIs, injection, or live user-window
movement are involved. Popup placement uses its dock's actual physical position, including mixed-DPI/negative-coordinate
monitors. The default theme slides the popup from the relevant dock edge and respects reduced-motion preferences.

Additional manual acceptance:

- Test browser, Explorer, NetEase Cloud Music and other apps, including several windows belonging to one app.
- Group more than 16 independent windows under one dock item, scroll to later windows, and confirm previews remain
  usable and the popup dismisses after leaving. Scroll back and verify that native thumbnails follow the visible cards.
- During a temporary peek, close the window that was immediately above the source or toggle its always-on-top state.
  Leave the preview and confirm the source returns behind the other surviving windows without taking focus.
- Switch Douyin videos repeatedly with the popup visible. The image should follow the actual source window without a
  title/resize event. Compare animation and video smoothness with the Windows taskbar, without assuming a fixed FPS.
- Hover a minimized window, leave without clicking, and confirm it is minimized again. Repeat with an already open
  window and confirm its stacking order is restored.
- Move onto the real window and click it; confirm the same window stays open. Repeat by clicking its thumbnail.
- Rapidly switch thumbnails, close a source, press Escape, change foreground apps, and exit Seelen during a temporary
  presentation. No old session should restore or activate the newly selected window.

Native tests use transparent offscreen fixture windows. Automated tests never move/focus a user's app or send playback
commands; desktop acceptance remains a manual step.
