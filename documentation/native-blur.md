# Experimental native background blur for themes

Themes can opt real elements into a native Gaussian blur of what is behind the widget. The blur is drawn with
`CreateBackdropBrush` and a Direct2D Gaussian blur in a Windows Composition target attached to the **existing widget
HWND**, below its transparent WebView2 child. It does not create another window, intercept input, change focus, or
change the WebView opacity. Nothing changes for themes that do not opt in: no Composition host is created and no timer
loop runs.

## Implementation map

- `src/background/widgets/native_blur.rs`: per-widget Composition host, clipped region visuals, resource reuse, document
  sessions, sequence and visibility-epoch checks, validation, popup switch and focus handling, disposal.
- `src/background/widgets/native_blur_effect.rs`: backdrop + Gaussian effect brush and the Composition dispatcher.
- `src/background/widgets/native_blur_visibility.rs`: pure visibility-epoch state machine (show/close/switch ordering).
- `src/background/widgets/native_blur_focus.rs`: pure, reversible focus-loss gate for popups.
- `libs/core/src/state/theme/native_blur.rs` and `handlers/commands.rs`: typed region/clip payloads and the generated
  commands (`BeginNativeBlurSession`, `SyncNativeBlurRegions`, `SetNativeBlurVisibility`).
- `libs/core/src/state/theme/nativeBlur.ts`: one controller per document: discovery, observers, animation tracking,
  batching through bounded lanes (`nativeBlurLane.ts`), fallback state and teardown.
- `nativeBlurGeometry.ts`, `nativeBlurAnimation.ts`, `nativeBlurProjection.ts`, `nativeBlurMotion.ts`,
  `nativeBlurCloseAnimation.ts`: CSS geometry, corner resolution, opacity curves and the opt-in flip outline.
- `libs/core/src/state/widget/abstractions/4_visibility.ts` and `hideAnimations.ts`: hide ordering (native exit first,
  then the HTML close animation, then the HWND hide).
- `src/ui/vanilla/entry-point/MainSetup.ts` and `libs/core/src/state/theme/theming.ts`: common widget bootstrap and
  theme refresh, so third-party widgets bundled with an older library also get the controller.

## CSS-only theme opt-in

Use an existing real element. Normal CSS determines its size, position and border radius:

```css
/* Example only, these declarations are not part of the default theme. */
.ft-bar {
  --slu-native-blur: 1;
  --slu-native-blur-amount: 18px;
  border-radius: 18px;
  background: rgb(20 24 32 / 25%);
  backdrop-filter: blur(18px); /* Optional fallback. */
}

.ft-bar[data-slu-native-blur-state="ready"] {
  backdrop-filter: none; /* Avoid applying both effects. */
}
```

`--slu-native-blur` is a registered, **non-inherited** number: `1` enables, `0` disables. `--slu-native-blur-amount` is
an inherited length, `18px` by default and clamped to `64px`; `0px` disables the region. The amount is the Gaussian
standard deviation, so it does not look identical to CSS `blur()` at the same number.

Widget authors can use `<div data-native-blur>...</div>` instead (`"false"` disables the marker).

The controller sets `data-slu-native-blur-state="ready"` on submitted elements and
`data-slu-native-blur-status="ready|idle|unavailable"` on the document root. Keep a fallback unless the element is
ready. Do not use these attributes to change layout, the opt-in property or visibility, which would create a feedback
loop.

### Optional animated outline

By default the native blur leaves as soon as a widget starts hiding, and only axis-aligned shapes are supported. A theme
that animates a popup with a single-plane `perspective` + `rotateX` can opt that element into a projected outline:

```css
.popup {
  --slu-native-blur-projection: flip; /* fit the blur to the projected rounded outline */
  --slu-native-blur-close: animate; /* keep following the outline during the close animation */
}
```

The outline is a convex polygon (up to 64 points) validated by the backend. Unsupported transforms fall back to no
native outline instead of blurring the bounding box. Focus loss and popup switches still remove the blur immediately.

## Lifecycle and synchronization

- One region per opted-in element: border box, four independent elliptical radii, viewport and `overflow` ancestor
  clipping, and the element/ancestor opacity product. `devicePixelRatio` converts CSS pixels to Composition pixels.
- Geometry updates are coalesced (at most 30 batches per second). Geometry and visibility each go through a bounded
  lane: one request in flight per visibility epoch and only the newest pending value kept, so a late acknowledgement can
  never hold back a newer show or close.
- Hiding a widget advances the visibility epoch and asks the backend to drop the blur **before** the HTML close
  animation starts, without waiting for an acknowledgement. A native root-opacity gate is armed with the close deadline,
  so the layer is gone even if the WebView thread stalls. A new show cancels older closes and stale geometry.
- Toolbar popup switch: the toolbar trigger is marked with `customArgs.__seelenToolbarToggle`. The backend clears the
  native blur of the other built-in popups on the UI thread before triggering the next one, and only the latest switch
  request is delivered. A widget can join this handshake with `nativeBlurSwitch: true` in its definition and by handling
  `widget::native-blur-close`.
- Focus loss: on a foreground change the backend hides the blur of built-in popups that lost focus directly (without
  waiting for the frontend focus debounce). It is reversible if focus returns to the popup or one of its owned windows
  before the popup is hidden.
- Navigation and window destruction invalidate the document session and release all resources. IPC is scoped to the
  calling WebView, no arbitrary HWND is accepted.

## Limitations

- **Native backgrounds are below all HTML**, not interleaved with DOM stacking contexts. Opaque HTML backgrounds cover
  them, and they cannot blur other HTML elements like `backdrop-filter`. Avoid overlapping or nested regions.
- Rotation (except the opt-in flip outline), skew, negative scale, fragmented inline boxes, `clip-path`, masks and
  filters are omitted. Pseudo-elements, SVG and shadow DOM are not supported. At most 16 clips per region.
- At most 32 regions, `64px` blur and 64 million backing pixels per widget.
- Browser and Composition clocks are separate: CSS and native layers can differ briefly during fast animations. This is
  meant for background panels, not a frame-perfect animation primitive.
- CSS-only selection needs `CSS.registerProperty` and readable stylesheet rules. CSSOM edits that do not go through a
  theme reload are only picked up after reloading the theme.

## Manual checks

Apply the example CSS to a transparent panel through a test theme, place a high-contrast window behind it, then check:
blur and sharp text, corner radii and overflow clipping, drag/resize, 100%/150%/200% scaling, fade in/out, reopening
during hide, switching toolbar popups quickly, clicking another app while a popup is open, theme reloads and widget
restarts. No blur residue may remain after a popup closes.
