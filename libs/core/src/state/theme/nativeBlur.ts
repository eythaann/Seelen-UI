/// <reference lib="dom.iterable" />

import { invoke } from "@tauri-apps/api/core";
import type { NativeBlurRegion } from "../../../gen/types/NativeBlurRegion.ts";
import { readNativeBlurGeometry } from "./nativeBlurGeometry.ts";
import { type OpacityPlan, readOpacityPlan } from "./nativeBlurAnimation.ts";

const ENABLE = "--slu-native-blur";
const AMOUNT = "--slu-native-blur-amount";
const STATUS = "data-slu-native-blur-status";
const ELEMENT_STATUS = "data-slu-native-blur-state";
const FRAME_INTERVAL = 1000 / 30;
const MAX_REGIONS = 32;

interface Frame {
  visibilityEpoch: number;
  scale: number;
  regions: NativeBlurRegion[];
}

/** Shared by all Widget frameworks through startThemingTool; no theme-side JS required. */
export function startNativeBlurController(): NativeBlurController {
  // Widget bundles may contain different copies of the library. The document owns one controller.
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  return runtime.__SLU_NATIVE_BLUR_CONTROLLER__ ??= new NativeBlurController();
}

/** Native deadlines are armed independently of geometry IPC and never delay HWND hide. */
export function beginNativeBlurClose(durationMs: number): number | undefined {
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  return runtime.__SLU_NATIVE_BLUR_CONTROLLER__?.beginClose(durationMs);
}

export function finishNativeBlurClose(epoch: number | undefined): void {
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  if (epoch !== undefined) void runtime.__SLU_NATIVE_BLUR_CONTROLLER__?.hide(epoch);
}

export function resumeNativeBlur(): void {
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  runtime.__SLU_NATIVE_BLUR_CONTROLLER__?.resume();
}

class NativeBlurController {
  private ids = new WeakMap<HTMLElement, number>();
  private nextId = 1;
  private elements: HTMLElement[] = [];
  private observed = new Set<Element>();
  private selectors: string[] = [];
  private selectorsDirty = true;
  private discoverDirty = true;
  private timer: ReturnType<typeof globalThis.setTimeout> | undefined;
  private frame = 0;
  private lastRead = 0;
  private lastSent = "";
  private session: number | undefined;
  private sessionRequest: Promise<number> | undefined;
  private visibilityEpoch = 0;
  private sequence = 0;
  private pending: Frame | undefined;
  private sending: Promise<void> | undefined;
  private suspended = false;
  private plans = new WeakMap<HTMLElement, { plan: OpacityPlan; id: number }>();
  private nextAnimationId = 1;
  private stopped = false;
  private failed = false;
  private propertiesSupported = false;
  private resizeObserver = new ResizeObserver(() => this.schedule());
  private mutationObserver: MutationObserver;

  constructor() {
    // Enable must not inherit, otherwise opting in a panel would opt in all its children.
    if (typeof CSS.registerProperty === "function") {
      for (
        const definition of [
          { name: ENABLE, syntax: "<number>", inherits: false, initialValue: "0" },
          { name: AMOUNT, syntax: "<length>", inherits: true, initialValue: "18px" },
        ]
      ) {
        try {
          CSS.registerProperty(definition);
        } catch { /* Already registered by this document. */ }
      }
      this.propertiesSupported = true;
    }
    this.mutationObserver = new MutationObserver((records) => {
      if (
        records.every((record) =>
          record.type === "attributes" &&
          (record.attributeName === STATUS || record.attributeName === ELEMENT_STATUS)
        )
      ) return;
      if (this.suspended || document.hidden) {
        // Reconcile once on resume, rather than traversing a hidden Widget's changing DOM.
        this.discoverDirty = true;
        this.selectorsDirty = true;
        return;
      }
      const stylesChanged = records.some((record) => {
        const target = record.target instanceof Element ? record.target : record.target.parentElement;
        return !!target?.closest("head, style, link[rel=stylesheet]") ||
          [...record.addedNodes, ...record.removedNodes].some((node) =>
            node instanceof Element &&
            (node.matches("style, link[rel=stylesheet]") || node.querySelector("style, link[rel=stylesheet]"))
          );
      });
      if (
        !stylesChanged && !this.elements.length && !this.selectors.length && !this.selectorsDirty &&
        !records.some((record) =>
          record.attributeName === "data-native-blur" ||
          (record.attributeName === "style" && record.target instanceof HTMLElement &&
            record.target.style.getPropertyValue(ENABLE)) ||
          [...record.addedNodes].some((node) =>
            node instanceof Element &&
            (node.matches('[data-native-blur], [style*="--slu-native-blur"]') ||
              node.querySelector('[data-native-blur], [style*="--slu-native-blur"]'))
          )
        )
      ) return;
      this.discoverDirty = true;
      if (stylesChanged) this.selectorsDirty = true;
      this.schedule();
    });
    this.mutationObserver.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      characterData: true,
    });
    document.addEventListener("scroll", this.schedule, true);
    document.addEventListener("visibilitychange", this.visibilityChanged);
    for (
      const event of [
        "transitionrun",
        "transitionend",
        "transitioncancel",
        "animationstart",
        "animationend",
        "animationcancel",
      ]
    ) {
      document.addEventListener(event, this.motionChanged, true);
    }
    document.addEventListener("pointerover", this.discover, true);
    document.addEventListener("pointerout", this.discover, true);
    document.addEventListener("focusin", this.discover, true);
    document.addEventListener("focusout", this.discover, true);
    document.addEventListener("load", this.refresh, true);
    globalThis.addEventListener("resize", this.discover);
    globalThis.addEventListener("pagehide", this.stop, { once: true });
    this.schedule();
  }

  refresh = (): void => {
    this.failed = false;
    this.selectorsDirty = true;
    this.discover();
  };

  beginClose(durationMs: number): number {
    const epoch = ++this.visibilityEpoch;
    void this.requestVisibility(epoch, false, Date.now() + Math.max(0, Math.min(durationMs, 2000)));
    this.cancelScheduledRead();
    // Capture the newly started CSS fade now, not at the next geometry polling interval.
    if (durationMs > 0 && (this.session !== undefined || this.sessionRequest)) this.read();
    return epoch;
  }

  async hide(epoch?: number): Promise<void> {
    epoch ??= ++this.visibilityEpoch;
    if (epoch !== this.visibilityEpoch) return;
    this.suspended = true;
    this.cancelScheduledRead();
    this.pending = undefined;
    await this.requestVisibility(epoch, false, Date.now());
    if (epoch !== this.visibilityEpoch) return;
    for (const element of this.elements) {
      if (element.hasAttribute(ELEMENT_STATUS)) element.removeAttribute(ELEMENT_STATUS);
    }
    this.lastSent = "";
    this.status("idle");
  }

  resume(): void {
    const epoch = ++this.visibilityEpoch;
    this.suspended = false;
    this.discoverDirty = true;
    this.plans = new WeakMap();
    void this.requestVisibility(epoch, true, Date.now());
    this.schedule();
  }

  private ensureSession(): Promise<number> {
    if (this.session !== undefined) return Promise.resolve(this.session);
    return this.sessionRequest ??= invoke<number>("begin_native_blur_session").then((session) => {
      this.session = session;
      return session;
    });
  }

  private async requestVisibility(epoch: number, visible: boolean, deadlineMs: number): Promise<void> {
    // Do not allocate a native session/host for themes that never used native blur.
    const request = this.sessionRequest ?? (this.session === undefined ? undefined : Promise.resolve(this.session));
    if (!request) return;
    try {
      const session = await request;
      if (epoch !== this.visibilityEpoch) return;
      // Separate control lane: a slow geometry request must not hold the hide signal hostage.
      await invoke("set_native_blur_visibility", { session, epoch, visible, deadlineMs });
    } catch (error) {
      if (!this.stopped && epoch === this.visibilityEpoch) this.fail(error);
    }
  }

  private discover = (): void => {
    if (!this.selectorsDirty && !this.selectors.length && !this.elements.length) return;
    this.discoverDirty = true;
    this.schedule();
  };

  private schedule = (): void => {
    if (this.stopped || this.failed || this.suspended || document.hidden || this.timer || this.frame) return;
    this.timer = globalThis.setTimeout(() => {
      this.timer = undefined;
      if (document.hidden) this.read();
      else this.frame = requestAnimationFrame(this.read);
    }, Math.max(0, FRAME_INTERVAL - (performance.now() - this.lastRead)));
  };

  private cancelScheduledRead(): void {
    clearTimeout(this.timer);
    cancelAnimationFrame(this.frame);
    this.timer = undefined;
    this.frame = 0;
  }

  private visibilityChanged = (): void => {
    if (document.hidden) {
      this.cancelScheduledRead();
      this.pending = undefined;
      void this.requestVisibility(++this.visibilityEpoch, false, Date.now());
    } else {
      if (!this.suspended) this.resume();
    }
  };

  private motionChanged = (event: Event): void => {
    if (
      this.stopped || this.failed || this.suspended || !(event.target instanceof Element) ||
      !this.observed.has(event.target)
    ) return;
    // Animation boundaries bypass the ordinary 30 Hz geometry throttle.
    clearTimeout(this.timer);
    this.timer = undefined;
    if (!this.frame) this.frame = requestAnimationFrame(this.read);
  };

  private collectSelectors(): void {
    const selectors = new Set<string>();
    const visit = (rules: CSSRuleList): void => {
      for (const rule of rules) {
        if (rule instanceof CSSStyleRule && rule.style.getPropertyValue(ENABLE)) selectors.add(rule.selectorText);
        if ("cssRules" in rule) visit((rule as CSSGroupingRule).cssRules);
        if (rule instanceof CSSImportRule) {
          try {
            if (rule.styleSheet) visit(rule.styleSheet.cssRules);
          } catch { /* Cross-origin import. */ }
        }
      }
    };
    for (const sheet of [...document.styleSheets, ...document.adoptedStyleSheets]) {
      try {
        visit(sheet.cssRules);
      } catch { /* Cross-origin sheets require explicit data-native-blur markers. */ }
    }
    this.selectors = [...selectors];
    this.selectorsDirty = false;
  }

  private findElements(): void {
    if (this.selectorsDirty) this.collectSelectors();
    const candidates = new Set(document.querySelectorAll<HTMLElement>("[data-native-blur]"));
    if (this.propertiesSupported) {
      for (const selector of [...this.selectors, '[style*="--slu-native-blur"]']) {
        try {
          for (const element of document.querySelectorAll<HTMLElement>(selector)) candidates.add(element);
        } catch { /* A selector for pseudo-elements has no independent native surface. */ }
      }
    }
    const elements = [...candidates].filter((element) => {
      if (!(element instanceof HTMLElement) || element.dataset.nativeBlur === "false") return false;
      return element.hasAttribute("data-native-blur") ||
        Number.parseFloat(getComputedStyle(element).getPropertyValue(ENABLE)) > 0;
    });
    for (const element of this.elements) if (!elements.includes(element)) element.removeAttribute(ELEMENT_STATUS);
    this.elements = elements;
    const observed = new Set<Element>();
    for (const element of this.elements) {
      for (let node: HTMLElement | null = element; node; node = node.parentElement) observed.add(node);
    }
    for (const node of this.observed) if (!observed.has(node)) this.resizeObserver.unobserve(node);
    for (const node of observed) if (!this.observed.has(node)) this.resizeObserver.observe(node);
    this.observed = observed;
    this.discoverDirty = false;
  }

  private read = (): void => {
    this.frame = 0;
    if (this.stopped || this.failed || this.suspended || document.hidden) return;
    this.lastRead = performance.now();
    try {
      if (this.discoverDirty) this.findElements();
      // Styles are live CSSStyleDeclaration objects, scoped to this synchronous read only.
      const styles = new Map<Element, CSSStyleDeclaration>();
      const readStyle = (element: Element): CSSStyleDeclaration => {
        let style = styles.get(element);
        if (!style) {
          style = getComputedStyle(element);
          styles.set(element, style);
        }
        return style;
      };
      const regions: NativeBlurRegion[] = [];
      if (!document.hidden && !this.suspended) {
        for (const element of this.elements) {
          const amount = Number.parseFloat(readStyle(element).getPropertyValue(AMOUNT) || "18");
          if (!Number.isFinite(amount) || amount <= 0) continue;
          const geometry = readNativeBlurGeometry(element, readStyle);
          if (!geometry) continue;
          if (!this.ids.has(element)) this.ids.set(element, this.nextId++);
          let cached = this.plans.get(element);
          const candidate = readOpacityPlan(element, cached?.plan, readStyle);
          if (!candidate) this.plans.delete(element);
          else if (candidate.key !== cached?.plan.key) {
            cached = { plan: candidate, id: this.nextAnimationId++ };
            this.plans.set(element, cached);
          }
          const opacityAnimation = candidate && cached
            ? {
              id: cached.id,
              startedAtMs: cached.plan.startedAtMs,
              durationMs: cached.plan.durationMs,
              values: cached.plan.values,
            }
            : null;
          regions.push({
            id: this.ids.get(element)!,
            amount: Math.min(amount, 64),
            ...geometry,
            // The native animation owns opacity until completion; geometry updates must not restart it.
            opacity: opacityAnimation ? opacityAnimation.values[0]! : geometry.opacity,
            opacityAnimation,
          });
          if (regions.length === MAX_REGIONS) break;
        }
      }
      this.pending = { visibilityEpoch: this.visibilityEpoch, scale: devicePixelRatio, regions };
      void this.send();
      // Follow only animations affecting an opted-in element or its ancestors.
      // There is no permanent render loop for static or unused themes.
      if (
        this.elements.length && !document.hidden && !this.suspended && document.getAnimations().some((animation) => {
          const target = (animation.effect as KeyframeEffect | null)?.target;
          return animation.playState === "running" && target instanceof Element &&
            this.elements.some((element) => target === element || target.contains(element));
        })
      ) this.schedule();
    } catch (error) {
      this.fail(error);
    }
  };

  private send(): Promise<void> {
    return this.sending ??= this.drain().finally(() => {
      this.sending = undefined;
    });
  }

  private async drain(): Promise<void> {
    if (this.failed) return;
    try {
      while (this.pending) {
        const frame = this.pending;
        this.pending = undefined;
        const key = JSON.stringify(frame);
        if (key === this.lastSent || (!frame.regions.length && this.session === undefined)) continue;
        const session = await this.ensureSession();
        if (this.stopped || this.suspended || frame.visibilityEpoch !== this.visibilityEpoch) continue;
        await invoke("sync_native_blur_regions", { session, sequence: ++this.sequence, ...frame });
        if (this.stopped || this.suspended || frame.visibilityEpoch !== this.visibilityEpoch) continue;
        this.lastSent = key;
        const ids = new Set(frame.regions.map((region) => region.id));
        for (const element of this.elements) {
          if (ids.has(this.ids.get(element)!)) {
            if (element.getAttribute(ELEMENT_STATUS) !== "ready") element.setAttribute(ELEMENT_STATUS, "ready");
          } else if (element.hasAttribute(ELEMENT_STATUS)) element.removeAttribute(ELEMENT_STATUS);
        }
        this.status(frame.regions.length ? "ready" : "idle");
      }
    } catch (error) {
      this.fail(error);
    }
  }

  private status(value: string): void {
    if (this.stopped || document.documentElement.getAttribute(STATUS) === value) return;
    document.documentElement.setAttribute(STATUS, value);
    this.schedule();
  }

  private fail(error: unknown): void {
    this.failed = true;
    this.pending = undefined;
    this.lastSent = "";
    for (const element of this.elements) element.removeAttribute(ELEMENT_STATUS);
    if (this.session !== undefined) {
      void invoke("sync_native_blur_regions", {
        session: this.session,
        visibilityEpoch: ++this.visibilityEpoch,
        sequence: ++this.sequence,
        scale: devicePixelRatio,
        regions: [],
      }).catch(() => {});
    }
    this.session = undefined;
    this.sessionRequest = undefined;
    this.status("unavailable");
    console.warn("Native blur unavailable; CSS fallback remains in control", error);
  }

  private stop = (): void => {
    this.stopped = true;
    this.cancelScheduledRead();
    this.resizeObserver.disconnect();
    this.mutationObserver.disconnect();
    for (const element of this.elements) element.removeAttribute(ELEMENT_STATUS);
    document.removeEventListener("scroll", this.schedule, true);
    document.removeEventListener("visibilitychange", this.visibilityChanged);
    for (
      const event of [
        "transitionrun",
        "transitionend",
        "transitioncancel",
        "animationstart",
        "animationend",
        "animationcancel",
      ]
    ) {
      document.removeEventListener(event, this.motionChanged, true);
    }
    document.removeEventListener("pointerover", this.discover, true);
    document.removeEventListener("pointerout", this.discover, true);
    document.removeEventListener("focusin", this.discover, true);
    document.removeEventListener("focusout", this.discover, true);
    document.removeEventListener("load", this.refresh, true);
    globalThis.removeEventListener("resize", this.discover);
    this.pending = undefined;
    void this.requestVisibility(++this.visibilityEpoch, false, Date.now());
  };
}
