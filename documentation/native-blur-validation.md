# Native blur: evidence and remaining validation

This is a draft architecture proposal, not a claim of release readiness or zero GPU cost. Local prototype measurements
below predate the latest-master port in this PR. The live GPU run also predates the close-timer correction; it is not
presented as a fresh benchmark of the PR head.

## Live on/off pilot (2026-10-23)

Windows build 26200, RTX 5070 Ti driver 32.0.16.1742, Ryzen 5 7500F (12 logical processors), WebView2 155.0.4283.62.
Win32_VideoController reported 2560x1440 at 210Hz; DPI/HDR were not measured. A DeskIn virtual display adapter was also
installed. Hardware acceleration was enabled; no disable-gpu argument was observed. This was an unoptimized debug
executable.

The same live desktop/binary and Aurora snapshot were used in both arms. Other native and CSS backdrop blur was
disabled; only the existing Toolbar .ft-bar used native 18 CSS px blur in the on arm. The Toolbar background color was
identical. Background applications/wallpaper were not frozen, clocks/power were not locked, and exact visible geometry
was not retrieved.

Order: off/on, on/off, off/on. Each trial had 15 seconds of warm-up and approximately 60 seconds of sampling: 35
rounds/trial, 210 rounds total. Process-tree and GPU WMI counters were read before waiting one second (actual cadence
about 1.7 seconds). Collection itself has overhead. The original theme resource path and enabled-theme order were
verified restored after the test.

| Pair | DWM 3D off | DWM 3D on | Difference (percentage points) | Seelen-tree CPU off / on |
| ---- | ---------: | --------: | -----------------------------: | -----------------------: |
| 1    |     0.343% |    1.229% |                         +0.886 |          6.355% / 6.485% |
| 2    |     0.486% |    1.200% |                         +0.714 |          6.340% / 6.272% |
| 3    |     0.314% |    1.314% |                         +1.000 |          6.061% / 6.073% |
| Mean |     0.381% |    1.248% |                         +0.867 |          6.252% / 6.277% |

These are arithmetic means of sampled utilization counters, **not GPU frame times**. DWM serves the whole desktop, so
its activity is not blur-only. Different engine percentages are not added together. Seelen-tree GPU counters rounded to
zero, which does not mean the effect is free. CPU includes all Seelen descendants and is normalized to 12 logical
processors.

CPU differences had no consistent direction. Private-memory differences were +53.258/-12.456/ +30.155 MiB across the
pairs and were affected by WebView2 reclamation; they are not per-effect allocation or leak measurements. Attributed GPU
memory is not physical resident VRAM. No relevant Application WER Event 1001 was found during this short window; not all
errors were checked.

WPR GPU capture failed with 0xc5585011 (insufficient system-performance profiling policy). No security/policy settings
were changed. ETW frame times, input-to-display latency, dropped frames and power measurements are unavailable. The
compact numeric dataset is in native-blur-results.json; private process/log data and copies of the theme are
deliberately excluded.

## Native resource and frontend regression evidence

Five debug CPU submission runs, each with four clips/region and 40 updates:

| Regions | Median total CPU submission for 40 updates |          Range | Brushes per run |
| ------- | -----------------------------------------: | -------------: | --------------: |
| 1       |                                   0.188 ms | 0.182–0.309 ms |               1 |
| 8       |                                   1.619 ms | 1.390–1.983 ms |               8 |
| 32      |                                   7.188 ms | 5.466–9.010 ms |              32 |

These are not GPU completion times. A separate hidden-HWND test balanced 4,000 region creations and drops in 500
lifecycle cycles. Private bytes at cycles 20/100/300/500 were 2,703,360/2,596,864/2,822,144/2,891,776; the higher final
value precludes a leak-free claim. Matching Drop counts do not prove all driver/COM/GPU allocations were reclaimed.

An 85ms CSS fade initially hid at approximately 80.959ms with opacity 0.0475412. Using a computed remaining duration as
a wall-clock timeout could win before Animation.finished. The correction retains the 200ms watchdog and uses completion
as the normal signal. Ten local regression rounds then passed 70/70 cases; opacity at hide was zero in all ten fade
cases. Readiness-based setup replaced a flaky fixed 150ms initialization wait; assertions were not relaxed. Browser
tests bundle the actual controller/Widget code but mock platform IPC and HWND behavior. They are not visual/GPU
acceptance tests.

## Reproduction

From the repository root, after normal dependency/core setup:

```powershell
cargo test -p seelen-ui --bin seelen-ui native_blur -- --test-threads=1
node scripts/native-blur-native-perf.mjs native-resources.json
node scripts/native-blur-native-perf.mjs native-lifecycle.json --lifecycle-only
# Requires an existing Playwright installation and installed Edge; no browser is downloaded.
# Set SLU_PLAYWRIGHT_PATH to its module path if it is not locally resolvable.
node scripts/native-blur-close-tests.mjs close-results.json
node scripts/native-blur-perf.mjs dom-results.json
```

Core helper tests run from libs/core with `deno test`. Native resource tests need Windows. The browser and native
scripts label their measurement boundaries in their output.

For live GPU reproduction, fix a target surface and blur amount, use identical material/foreground in both arms with CSS
fallback blur disabled, warm up, alternate repeated on/off trials and record Seelen/WebView2/DWM separately. Never
report summed utilization across engines as total GPU usage. Keep other desktop activity constant and record hardware,
driver, display, WebView2, build and theme configuration. Use ETW/WPA/GPUView for frame/engine attribution rather than
these pilot counters.

Still outstanding: other GPU vendors/drivers, optimized-build comparison, long-running real Widgets, multiple
displays/mixed DPI, HDR, background motion, device-loss/suspend recovery, real popup latency, and actual WebView2
hardware-acceleration A/B. Daily local use suggests residue is much less noticeable, not that every remaining
synchronization edge has been eliminated.
