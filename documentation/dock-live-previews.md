# Live dock window previews

Hover a running dock app for 350 ms to open its previews without taking keyboard focus. Click a thumbnail to activate
that exact window; close and middle-click remain separate actions. The popup follows its dock side and monitor DPI.

Windows DWM draws live frames directly. Only layout, opacity and a lightweight lease/pointer heartbeat cross the webview
boundary. Requests are serialized and coalesced; unchanged layout is sent at most every 80 ms. At most 16 visible
thumbnails are registered, with priority for the hovered card. Scroll clipping maps into the original source crop
without stretching or covering the scrollbar. Missing native surfaces use the existing snapshot cache, with deduplicated
capture requests limited to four per second for visible fallbacks.

Hover a thumbnail for 450 ms to temporarily present its source without keyboard focus. Move onto the real window to
inspect it, or click either surface to keep it. Leaving both regions restores the previous stacking/minimized state.
PID-checked neighboring windows anchor restoration even if a neighbor closes. Leases reject cancelled late requests and
restore temporary state when the popup hides, the heartbeat expires, another app gains focus, or Seelen exits. Cleanup
stays behind the platform contract and does not initialize unused native preview managers.

## Checks

```text
npm test
npm run type-check
npm run build:ui
cargo test -p seelen-core --features gen-binds --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test -p seelen-ui live_previews::tests -- --include-ignored --test-threads=1
cargo test -p seelen-ui window_peek::tests -- --test-threads=1
```

Regenerate core bindings/package after command or data changes. DWM fixture tests require an interactive Windows
desktop; they create only offscreen fixtures and never operate user applications.

Manual acceptance: test every dock side, mixed DPI/negative-coordinate monitors, typing while hovering, minimized and
topmost windows, more than 16 grouped windows with partly clipped cards, rapid source changes, Escape, popup hide,
foreground changes, and Seelen exit during a peek. Verify restoration when a saved neighbor closes. Protected video and
minimized windows can provide a frozen/unavailable DWM frame; compare smoothness without claiming a fixed FPS.

This feature does not add media-session controls, media shortcuts, or media settings.
