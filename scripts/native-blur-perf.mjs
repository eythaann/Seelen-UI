// Isolated Chromium DOM benchmark. Tauri IPC is mocked, NOT a native GPU benchmark.
// NODE_PATH must resolve Playwright; no browser/profile belonging to the user is used.
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { buildSync } from "esbuild";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.SLU_PLAYWRIGHT_PATH || "playwright");
const output = process.argv[2];
if (!output) throw new Error("Usage: node scripts/native-blur-perf.mjs <result.json> [--disable-gpu]");
const source = "libs/core/src/state/theme/nativeBlur.ts";
const bundle = buildSync({
  stdin: {
    contents: readFileSync(source, "utf8").replace(
      'import { invoke } from "@tauri-apps/api/core";',
      "const invoke = (...args) => globalThis.__blurInvoke(...args);",
    ),
    resolveDir: resolve("libs/core/src/state/theme"),
    loader: "ts",
  },
  bundle: true,
  write: false,
  format: "iife",
  globalName: "BlurTest",
  platform: "browser",
}).outputFiles[0].text;
const disabled = process.argv.includes("--disable-gpu");
const browser = await chromium.launch({
  channel: "msedge",
  headless: true,
  args: ["--js-flags=--expose-gc", ...(disabled ? ["--disable-gpu", "--disable-gpu-compositing"] : [])],
});
const result = {
  timestamp: new Date().toISOString(),
  browser: browser.version(),
  hardwareMode: disabled ? "Chromium --disable-gpu (NOT WebView2)" : "Chromium default (NOT WebView2)",
  sourceSha256: createHash("sha256").update(readFileSync(source)).digest("hex"),
  note: "Real DOM/layout and controller; mocked 10 ms IPC; no Composition visuals or GPU timing.",
  cases: [],
};
try {
  for (
    const scenario of [
      "static",
      "geometry-1",
      "geometry-8",
      "geometry-32",
      "suspended",
      "document-hidden",
      "removed",
      "disabled",
      "opacity-animation",
      "theme-churn",
      "lifecycle-stress",
      "backpressure",
    ]
  ) {
    const page = await browser.newPage({ viewport: { width: 1000, height: 800 } });
    try {
      page.on("pageerror", (error) => console.error(`page error: ${error.message}`));
      page.on("console", (message) => {
        if (message.type() === "warning") console.error(message.text());
      });
      await page.setContent(`<style>
        .panel { --slu-native-blur:1; --slu-native-blur-amount:18px; position:absolute;
          top:20px; left:20px; width:120px; height:80px; border-radius:14px; }
        .outer { overflow:hidden; width:900px; height:700px; }
        </style><div class="outer" id="panels"></div><span id="noise">0</span>`);
      await page.evaluate(() => {
        globalThis.metrics = {
          queries: 0,
          styles: 0,
          rects: 0,
          readyWrites: 0,
          syncs: 0,
          bytes: 0,
          maxFlight: 0,
          active: 0,
        };
        globalThis.ipcDelay = 10;
        let flight = 0;
        let visibilityEpoch = 0, deadline = Infinity;
        const clear = () => {
          globalThis.metrics.active = 0;
          globalThis.lastRegions = [];
        };
        globalThis.__blurInvoke = async (command, payload) => {
          if (command === "begin_native_blur_session") return 1;
          if (command === "set_native_blur_visibility") {
            if (payload.epoch < visibilityEpoch) return;
            if (payload.visible) {
              if (payload.epoch > visibilityEpoch || deadline === Infinity) {
                visibilityEpoch = payload.epoch;
                deadline = Infinity;
              }
            } else {
              deadline = payload.epoch === visibilityEpoch
                ? Math.min(deadline, payload.deadlineMs)
                : payload.deadlineMs;
              visibilityEpoch = payload.epoch;
              const expected = visibilityEpoch;
              setTimeout(() => {
                if (visibilityEpoch === expected && Date.now() >= deadline) clear();
              }, Math.max(0, deadline - Date.now()));
              if (Date.now() >= deadline) clear();
            }
            return;
          }
          const m = globalThis.metrics;
          m.syncs++;
          m.bytes += JSON.stringify(payload).length;
          m.maxFlight = Math.max(m.maxFlight, ++flight);
          await new Promise((resolve) => setTimeout(resolve, globalThis.ipcDelay));
          if (payload.visibilityEpoch >= visibilityEpoch) {
            if (payload.visibilityEpoch > visibilityEpoch) {
              visibilityEpoch = payload.visibilityEpoch;
              deadline = Infinity;
            }
            if (Date.now() < deadline) {
              m.active = payload.regions.length;
              globalThis.lastRegions = payload.regions;
            }
          }
          flight--;
        };
        const query = Document.prototype.querySelectorAll;
        Document.prototype.querySelectorAll = function (...args) {
          globalThis.metrics.queries++;
          return query.apply(this, args);
        };
        const style = globalThis.getComputedStyle;
        globalThis.getComputedStyle = (...args) => {
          globalThis.metrics.styles++;
          return style(...args);
        };
        const rect = Element.prototype.getBoundingClientRect;
        Element.prototype.getBoundingClientRect = function () {
          globalThis.metrics.rects++;
          return rect.call(this);
        };
        const attr = Element.prototype.setAttribute;
        Element.prototype.setAttribute = function (name, value) {
          if (name === "data-slu-native-blur-state") globalThis.metrics.readyWrites++;
          return attr.call(this, name, value);
        };
        globalThis.createPanels = (count) => {
          document.getElementById("panels").replaceChildren(...Array.from({ length: count }, (_, i) => {
            const panel = document.createElement("div");
            panel.className = "panel";
            panel.style.left = `${20 + (i % 8) * 100}px`;
            panel.style.top = `${20 + Math.floor(i / 8) * 90}px`;
            return panel;
          }));
        };
      });
      await page.addScriptTag({ content: bundle });
      const count = scenario.startsWith("geometry-") ? Number(scenario.split("-")[1]) : 8;
      await page.evaluate((count) => {
        globalThis.createPanels(count);
        BlurTest.startNativeBlurController();
      }, count);
      await page.waitForFunction((count) => globalThis.metrics.active === count, count, { timeout: 10000 });
      await page.waitForTimeout(150); // Drain one-off status/layout notifications before measuring steady state.
      const initialActive = await page.evaluate(() => globalThis.metrics.active);
      if (initialActive !== count) throw new Error(`Initialization failed: ${scenario}, ${initialActive}/${count}`);
      const cdp = await page.context().newCDPSession(page);
      await cdp.send("HeapProfiler.collectGarbage");
      const heapBefore = (await cdp.send("Runtime.getHeapUsage")).usedSize;
      const data = await page.evaluate(async (scenario) => {
        const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        const noise = document.getElementById("noise");
        const panels = document.getElementById("panels");
        const controller = BlurTest.startNativeBlurController();
        if (scenario === "suspended") await controller.hide();
        if (scenario === "document-hidden") {
          Object.defineProperty(document, "hidden", { configurable: true, value: true });
          document.dispatchEvent(new Event("visibilitychange"));
          await wait(100);
        }
        globalThis.gc?.();
        for (const key of Object.keys(globalThis.metrics)) if (key !== "active") globalThis.metrics[key] = 0;
        const start = performance.now();
        let lifecycleEmpty = true;
        if (scenario === "static") await wait(1000);
        else if (scenario.startsWith("geometry-")) {
          for (let i = 0; i < 40; i++) {
            panels.firstElementChild.style.transform = `translateX(${i + 1}px)`;
            await wait(45);
          }
        } else if (scenario === "suspended" || scenario === "document-hidden") {
          for (let i = 0; i < 100; i++) {
            noise.textContent = String(i);
            await wait(10);
          }
        } else if (scenario === "removed") {
          panels.replaceChildren();
          await wait(150);
        } else if (scenario === "disabled") {
          for (const element of panels.children) element.style.setProperty("--slu-native-blur", "0");
          await wait(150);
          if (globalThis.metrics.active !== 0) throw new Error("Opt-out left native regions alive");
        } else if (scenario === "opacity-animation") {
          panels.style.transition = "opacity 180ms ease-out";
          panels.style.opacity = "0";
          await wait(250);
          if (globalThis.metrics.active !== 0) throw new Error("Fade left native regions alive");
          panels.style.opacity = "1";
          await wait(250);
          if (globalThis.metrics.active !== 8) throw new Error("Fade-in lost native regions");
        } else if (scenario === "theme-churn") {
          const style = document.createElement("style");
          document.head.append(style);
          for (let i = 0; i < 50; i++) {
            style.textContent = `.panel { color: ${i % 2 ? "red" : "blue"}; }`;
            controller.refresh();
            await wait(40);
          }
          style.remove();
          await wait(100);
        } else if (scenario === "lifecycle-stress") {
          for (let i = 0; i < 100; i++) {
            globalThis.createPanels(32);
            controller.refresh();
            await wait(80);
            panels.replaceChildren();
            await wait(80);
            lifecycleEmpty &&= globalThis.metrics.active === 0;
          }
        } else if (scenario === "backpressure") {
          globalThis.ipcDelay = 100;
          for (let i = 0; i < 120; i++) {
            panels.firstElementChild.style.left = `${i}px`;
            await wait(3);
          }
          await wait(350);
        }
        await wait(150);
        const elapsedMs = performance.now() - start;
        const measured = { ...globalThis.metrics };
        const geometrySignature = JSON.stringify(
          globalThis.lastRegions.map(({ id, amount, clips, opacity }) => ({ id, amount, clips, opacity })),
        );
        if (scenario === "suspended" || scenario === "document-hidden") {
          if (scenario === "suspended") controller.resume();
          else {
            Object.defineProperty(document, "hidden", { configurable: true, value: false });
            document.dispatchEvent(new Event("visibilitychange"));
          }
          await wait(150);
          if (globalThis.metrics.active !== 8) throw new Error("Resume lost native blur regions");
        }
        panels.replaceChildren();
        await wait(150);
        globalThis.gc?.();
        return {
          scenario,
          elapsedMs,
          ...measured,
          lifecycleEmpty,
          cleanupActive: globalThis.metrics.active,
          geometrySignature,
        };
      }, scenario);
      await cdp.send("HeapProfiler.collectGarbage");
      data.heapBefore = heapBefore;
      data.heapAfter = (await cdp.send("Runtime.getHeapUsage")).usedSize;
      data.geometrySha256 = createHash("sha256").update(data.geometrySignature).digest("hex");
      delete data.geometrySignature;
      if (data.cleanupActive || !data.lifecycleEmpty || data.maxFlight > 1) {
        throw new Error(`Lifecycle assertion failed: ${JSON.stringify(data)}`);
      }
      result.cases.push(data);
      console.log(JSON.stringify(data));
    } finally {
      await page.close();
    }
  }
} finally {
  await browser.close();
}
writeFileSync(output, JSON.stringify(result, null, 2) + "\n");
