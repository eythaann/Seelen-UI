# Experimental native background blur for themes

This opt-in prototype uses Windows Composition (`CreateBackdropBrush` and a Direct2D Gaussian effect) inside the
existing Widget HWND, underneath its transparent WebView2 child. It does not create another production window, intercept
input, change focus, or alter WebView opacity. Default theme appearance is unchanged. Theme authors should still follow
Seelen's surface and translucency design guidelines.

## Theme API

Apply the properties to an existing real element in the target Widget stylesheet:

```css
.ft-bar {
  --slu-native-blur: 1;
  --slu-native-blur-amount: 18px;
  border-radius: 18px;
  background: rgb(20 24 32 / 25%);
  backdrop-filter: blur(18px); /* Optional CSS fallback. */
}

.ft-bar[data-slu-native-blur-state="ready"] {
  backdrop-filter: none; /* Avoid running both blur implementations. */
}
```

`--slu-native-blur` is a registered, non-inherited number: `1` enables, `0` disables. `--slu-native-blur-amount` is an
inherited CSS length, defaults to `18px`, and is clamped to `64px`; `0px` disables a region. The amount is Gaussian
standard deviation, not a promise of matching Acrylic or CSS blur at the same number. Widget authors may alternatively
mark an element with `data-native-blur`; `data-native-blur="false"` disables that marker. CSS cannot create attributes,
so CSS-only themes use the custom properties.

The controller reserves `data-slu-native-blur-state="ready"` on submitted elements, and
`data-slu-native-blur-status="ready|idle|unavailable"` on the document root. Ready means native resource submission
succeeded, not that pixels were verified on the user's GPU. Keep a fallback. Do not use readiness to change layout,
visibility, or opt-in properties: that creates a feedback loop. Removing the opt-in properties releases native resources
without changing the original DOM.

## Architecture and lifecycle

- Rust owns per-document sessions, the Composition target, clipped visuals, brushes and effects. Commands are generated
  through the existing core command declarations, scoped to the caller's WebView; callers cannot supply an arbitrary
  HWND.
- A shared document-level controller discovers opted-in elements and synchronizes CSS geometry. Both normal library
  startup and the shared page bootstrap use a singleton, including Widget bundles built with an older library. Theme
  reloads refresh the same controller.
- Geometry snapshots are coalesced at up to 30 batches/second, deduplicated, with one geometry request in flight and
  only the latest pending frame. Static/unused themes have no permanent animation loop. A mutation observer still
  detects later opt-ins.
- Brushes are reused until blur strength changes. Clip paths are updated only when their shapes change. Empty frames
  drop the host; navigation and destruction invalidate the session.
- Simple normal, single-iteration, two-keyframe opacity animations are bridged to Composition. Supported easing is
  linear, standard easing or cubic-bezier, with positive playback rate and replace composition, for up to 2 seconds.
  Unsupported timing uses opacity snapshots instead.
- Widget close waits only newly started/resumed/retimed finite animations. `Animation.finished` handles normal
  completion and cancellation; the original 200ms limit is only a watchdog, not an estimated animation deadline.
  Repeated hide calls share the current operation.
- Independent visibility epochs reject stale geometry, closes and cleanup tasks after reopening. Native root opacity is
  cut off at the watchdog, and native cleanup releases the host. Normal completion requests immediate cleanup and HWND
  hide without waiting for blur IPC. Initial delivery still depends on IPC/the native UI thread; there is no
  frame-atomic guarantee.

## Geometry and limits

Supported: element border boxes, independent elliptical corner radii with CSS normalization, ancestor padding-box
overflow clips, viewport clipping, axis-aligned positive scale/translation, and the product of element/ancestor opacity.
Loss of focus does not disable blur. CSS pixels are converted using devicePixelRatio.

Native backgrounds are below **all HTML**, not interleaved with arbitrary DOM stacking contexts. Opaque HTML covers
them. They cannot sample other HTML as if this were CSS backdrop-filter. Avoid overlapping/nested native regions, which
can compound blur.

Rotation, skew, perspective, negative scale, fragmented inline boxes, masks, filters, clip-path, pseudo-elements, SVG
and shadow DOM are not independently reproduced. Unsupported geometry is omitted rather than overdrawn; retain the CSS
fallback. Fast movement can briefly diverge from WebView2 rendering because geometry and native composition are
asynchronous.

The limits per Widget are 32 visible regions, 16 clips per region including the viewport, 64 CSS px blur, and 64 million
backing pixels of total region area. These are safety bounds, not performance targets. Prefer a few background panels
over many overlapping regions; do not animate blur amount unless necessary, because changing it rebuilds the effect
brush.

CSS-only discovery requires CSS.registerProperty and accessible stylesheet rules. Direct CSSOM insertRule changes
without a DOM/theme event need a theme refresh. Background visibility and shared hide helpers are supported; a
third-party Widget that bypasses them may need integration.

## Validation and review questions

See [validation evidence and limitations](native-blur-validation.md). The local prototype has been visually exercised
with rounded corners and loss of focus; that is not broad GPU certification. No default-theme opt-in, Acrylic/Mica hook,
standalone test panel or theme-specific animation is included.

Feedback is requested on the CSS API, suitable Widget surfaces, native resource ownership, the shared bootstrap
compatibility approach and the cost/maintenance tradeoff. Cross-vendor GPU, optimized-build, suspend/resume, device-loss
and detailed ETW frame timing remain follow-up work.
