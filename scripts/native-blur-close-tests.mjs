// Real Widget_4 + DOM + Controller; mocked platform IPC/window. No user UI is touched.
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { build } from "esbuild";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.SLU_PLAYWRIGHT_PATH || "playwright");
const bundled = await build({
  stdin: {
    contents: `export { Widget_4 } from './libs/core/src/state/widget/abstractions/4_visibility.ts';
    export { startNativeBlurController } from './libs/core/src/state/theme/nativeBlur.ts';`,
    resolveDir: resolve("."),
    loader: "ts",
  },
  bundle: true,
  write: false,
  format: "iife",
  globalName: "CloseTest",
  platform: "browser",
  plugins: [{
    name: "test-platform-only",
    setup(build) {
      build.onResolve({ filter: /^\.\/3_autosize\.ts$/ }, () => ({ path: "parent", namespace: "test" }));
      build.onResolve({ filter: /^@tauri-apps\/api\/(window|core)$/ }, ({ path }) => ({ path, namespace: "test" }));
      build.onLoad({ filter: /.*/, namespace: "test" }, ({ path }) => ({
        contents: path === "parent"
          ? "export class Widget_3 { window = globalThis.testWindow; }"
          : path.endsWith("/core")
          ? "export const invoke = (...args) => globalThis.testInvoke(...args);"
          : "export const getCurrentWindow = () => globalThis.testWindow;",
        loader: "js",
      }));
    },
  }],
});
const browser = await chromium.launch({ channel: "msedge", headless: true });
const results = [];
try {
  for (
    const name of [
      "blocked-ipc",
      "late-close-after-reopen",
      "deadline-no-finish",
      "content-animation",
      "repeated-hide-reopen",
      "no-native-theme",
      "long-close-watchdog",
    ]
  ) {
    const page = await browser.newPage();
    try {
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.setContent(`<style>
        .panel { --slu-native-blur:1; --slu-native-blur-amount:18px; width:200px;height:100px;
          border-radius:18px; opacity:1; transition:opacity 85ms linear; }
        html[data-widget-hidden] .panel { opacity:0; }
        </style><div class="panel"></div><span id="noise">content</span>`);
      await page.evaluate((name) => {
        const mock = globalThis.mock = {
          epoch: 0,
          deadline: Infinity,
          active: 0,
          controls: [],
          frames: [],
          hideCalls: 0,
          block: false,
          closeDelay: 0,
          beginCalls: 0,
        };
        const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        globalThis.testWindow = {
          show: async () => {},
          hide: () => {
            mock.hideCalls++;
            mock.hideObservation = {
              time: performance.now(),
              opacity: getComputedStyle(document.querySelector(".panel")).opacity,
              animations: document.getAnimations().map((animation) => ({
                playState: animation.playState,
                currentTime: animation.currentTime,
                endTime: animation.effect?.getComputedTiming().endTime,
              })),
            };
            return Promise.resolve();
          },
          isVisible: () => Promise.resolve(true),
          close: async () => {},
        };
        globalThis.testInvoke = async (command, payload) => {
          if (command === "begin_native_blur_session") {
            mock.beginCalls++;
            return 1;
          }
          if (command === "set_native_blur_visibility") {
            mock.controls.push({ ...payload });
            const delay = payload.visible ? 0 : mock.closeDelay;
            if (mock.block) return new Promise(() => {});
            if (delay) await wait(delay);
            if (payload.epoch < mock.epoch) return;
            if (payload.visible) {
              if (payload.epoch > mock.epoch || mock.deadline === Infinity) {
                mock.epoch = payload.epoch;
                mock.deadline = Infinity;
              }
            } else {
              mock.deadline = payload.epoch === mock.epoch
                ? Math.min(mock.deadline, payload.deadlineMs)
                : payload.deadlineMs;
              mock.epoch = payload.epoch;
              const expected = mock.epoch;
              setTimeout(() => {
                if (mock.epoch === expected && Date.now() >= mock.deadline) mock.active = 0;
              }, Math.max(0, mock.deadline - Date.now()));
              if (Date.now() >= mock.deadline) mock.active = 0;
            }
          } else if (command === "sync_native_blur_regions") {
            mock.frames.push(payload);
            if (mock.block) return new Promise(() => {});
            if (payload.visibilityEpoch < mock.epoch) return;
            if (payload.visibilityEpoch > mock.epoch) {
              mock.epoch = payload.visibilityEpoch;
              mock.deadline = Infinity;
            }
            if (Date.now() < mock.deadline) mock.active = payload.regions.length;
          } else throw new Error(`Unexpected command: ${command}`);
        };
        if (name === "no-native-theme") document.querySelector(".panel").style.setProperty("--slu-native-blur", "0");
      }, name);
      await page.addScriptTag({ content: bundled.outputFiles[0].text });
      await page.evaluate(() => {
        globalThis.controller = CloseTest.startNativeBlurController();
        globalThis.widget = new CloseTest.Widget_4();
      });
      if (name === "no-native-theme") await page.waitForTimeout(150);
      else await page.waitForFunction(() => globalThis.mock.active === 1, null, { timeout: 10000 });
      const result = await page.evaluate(async (name) => {
        const assert = (condition, message) => {
          if (!condition) throw new Error(message);
        };
        const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        const panel = document.querySelector(".panel"), mock = globalThis.mock;
        const before = performance.now();
        if (name === "blocked-ipc") {
          panel.style.transition = "none";
          mock.block = true;
          panel.style.transform = "translateX(1px)";
          await wait(50);
          const done = await Promise.race([widget.hide().then(() => true), wait(75).then(() => false)]);
          assert(done && mock.hideCalls === 1, "Window hide incorrectly awaited stalled native IPC");
          assert(mock.controls.some((command) => !command.visible), "Close never entered control lane");
        } else if (name === "late-close-after-reopen") {
          panel.style.transition = "none";
          mock.closeDelay = 150;
          await widget.hide();
          await widget.show();
          await wait(250);
          assert(
            mock.active === 1 && !document.documentElement.hasAttribute("data-widget-hidden"),
            "Late close erased reopened panel",
          );
        } else if (name === "deadline-no-finish") {
          controller.beginClose(40);
          await wait(85);
          assert(mock.active === 0, "Native deadline did not clear without frontend finish");
          const frame = mock.frames.at(-1);
          await testInvoke("sync_native_blur_regions", { ...frame, visibilityEpoch: mock.epoch, sequence: 99999 });
          assert(mock.active === 0, "A late same-generation frame resurrected expired blur");
        } else if (name === "content-animation") {
          const unrelated = document.getElementById("noise").animate([{ opacity: 1 }, { opacity: .5 }], {
            duration: 1500,
          });
          await wait(20);
          await widget.hide();
          assert(mock.hideCalls === 1 && unrelated.playState === "running", "Close waited for unrelated content");
          assert(Number(getComputedStyle(panel).opacity) < .01, "Popup's own fade was skipped");
        } else if (name === "repeated-hide-reopen") {
          const first = widget.hide(), second = widget.hide();
          assert(first === second, "Repeated hide started a separate operation");
          await wait(20);
          await widget.show();
          await first;
          await wait(150);
          assert(mock.hideCalls === 0 && mock.active === 1, "Cancelled hide affected new show");
        } else if (name === "long-close-watchdog") {
          panel.style.transitionDuration = "1000ms";
          getComputedStyle(panel).opacity;
          const closed = await Promise.race([widget.hide().then(() => true), wait(600).then(() => false)]);
          assert(closed && mock.hideCalls === 1, "Close watchdog did not hide the window");
          assert(Number(mock.hideObservation.opacity) > .1, "Long transition was not bounded by the watchdog");
        } else {
          panel.style.transition = "none";
          await widget.hide();
          assert(mock.beginCalls === 0 && mock.controls.length === 0, "Non-native theme allocated a native session");
        }
        return {
          name,
          elapsedMs: performance.now() - before,
          hideCalls: mock.hideCalls,
          controlMessages: mock.controls.length,
          hideObservation: mock.hideObservation,
          passed: true,
        };
      }, name);
      if (errors.length) throw new Error(errors.join("\n"));
      results.push(result);
      console.log(JSON.stringify(result));
    } catch (error) {
      const diagnostic = await page.evaluate(() => globalThis.mock?.hideObservation).catch(() => null);
      const result = {
        name,
        passed: false,
        error: String(error.message).split("\n").slice(0, 3).join("\n"),
        diagnostic,
      };
      results.push(result);
      console.log(JSON.stringify(result));
    } finally {
      await page.close();
    }
  }
} finally {
  await browser.close();
}
if (process.argv[2]) {
  writeFileSync(
    process.argv[2],
    JSON.stringify(
      {
        timestamp: new Date().toISOString(),
        note: "Actual frontend code; mocked native/window IPC, not GPU pixels or a JS-stall GPU test.",
        results,
      },
      null,
      2,
    ) + "\n",
  );
}
if (results.some((result) => !result.passed)) process.exitCode = 1;
