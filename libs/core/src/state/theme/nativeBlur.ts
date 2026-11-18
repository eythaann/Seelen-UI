/// <reference lib="dom.iterable" />

import { invoke } from "@tauri-apps/api/core";
import type { NativeBlurRegion } from "../../../gen/types/NativeBlurRegion.ts";
import type { NativeBlurCloseAnimation } from "../../../gen/types/NativeBlurCloseAnimation.ts";
import { readCloseOutlines } from "./nativeBlurCloseAnimation.ts";
import { readNativeBlurMotion } from "./nativeBlurMotion.ts";
import { readNativeBlurGeometry } from "./nativeBlurGeometry.ts";
import { type OpacityPlan, readOpacityPlan } from "./nativeBlurAnimation.ts";
import { NativeBlurLane } from "./nativeBlurLane.ts";

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
  animateClose: boolean;
  motion?: NativeBlurCloseAnimation;
}

interface VisibilityControl {
  request: Promise<number>;
  epoch: number;
  visible: boolean;
  deadlineMs: number;
  closeAnimation?: NativeBlurCloseAnimation;
}

function usesAnimatedClose(style: CSSStyleDeclaration): boolean {
  return style.getPropertyValue("--slu-native-blur-projection").trim() === "flip" &&
    style.getPropertyValue("--slu-native-blur-close").trim() === "animate";
}

/** Shared by all Widget frameworks through startThemingTool; no theme-side JS required. */
export function startNativeBlurController(): NativeBlurController {
  // Widget bundles may contain different copies of the library. The document owns one controller.
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  return runtime.__SLU_NATIVE_BLUR_CONTROLLER__ ??= new NativeBlurController();
}

/** Native deadlines are armed independently of geometry IPC and never delay HWND hide. */
export function beginNativeBlurClose(durationMs: number, startAnimation: () => void): number | undefined {
  const runtime = globalThis as typeof globalThis & { __SLU_NATIVE_BLUR_CONTROLLER__?: NativeBlurController };
  const controller = runtime.__SLU_NATIVE_BLUR_CONTROLLER__;
  if (controller) return controller.beginClose(durationMs, startAnimation);
  startAnimation();
  return undefined;
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
  private geometryLane = new NativeBlurLane<Frame>((frame) => this.dispatch(frame), (error) => this.fail(error));
  private visibilityLane = new NativeBlurLane<VisibilityControl>(
    (control) => this.dispatchVisibility(control),
    (error) => this.fail(error),
  );
  private suspended = false;
  private plans = new WeakMap<HTMLElement, { plan: OpacityPlan; id: number }>();
  private nextAnimationId = 1;
  private motion?: { animation: Animation; plan?: NativeBlurCloseAnimation; armed: boolean };
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
      if (
        records.some((record) =>
          record.attributeName === "data-slu-native-blur-motion" &&
          record.target instanceof Element && record.target.hasAttribute("data-slu-native-blur-motion")
        )
      ) {
        this.cancelScheduledRead();
        this.frame = requestAnimationFrame(this.read);
        return;
      }
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

  beginClose(durationMs: number, startAnimation: () => void): number {
    const epoch = this.advanceEpoch();
    this.cancelScheduledRead();
    const animatedClose = this.elements.some((element) => usesAnimatedClose(getComputedStyle(element)));
    if (!animatedClose) {
      this.suspended = true;
      this.pending = undefined;
      this.lastSent = "";
      for (const element of this.elements) element.removeAttribute(ELEMENT_STATUS);
      this.status("idle");
      this.requestVisibility(epoch, false, Date.now());
      startAnimation();
      return epoch;
    }
    const deadline = Date.now() + Math.max(0, Math.min(durationMs, 2000));
    const element = animatedClose && this.elements.length === 1 ? this.elements[0] : undefined;
    const regionId = element && this.ids.get(element);
    let flat: ReturnType<typeof readNativeBlurGeometry>;
    try {
      flat = regionId && element ? readNativeBlurGeometry(element) : null;
    } catch {
      flat = null;
    }
    startAnimation();
    let closeAnimation: NativeBlurCloseAnimation | undefined;
    if (flat && element && regionId) {
      try {
        const opacity = readOpacityPlan(element);
        const outlines = opacity && opacity.values.at(-1) === 0
          ? readCloseOutlines(element, flat.clips[0]!, opacity.durationMs)
          : undefined;
        if (outlines && opacity) {
          closeAnimation = {
            regionId,
            scale: devicePixelRatio,
            outlines,
            opacity: {
              id: this.nextAnimationId++,
              startedAtMs: opacity.startedAtMs,
              durationMs: opacity.durationMs,
              values: opacity.values,
            },
          };
        }
      } catch {
        // Unsupported computed CSS falls back to the existing live geometry path.
      }
    }
    this.requestVisibility(epoch, false, deadline, closeAnimation);
    // Capture the newly started CSS fade now, not at the next geometry polling interval.
    if (durationMs > 0 && (this.session !== undefined || this.sessionRequest)) this.read();
    return epoch;
  }

  hide(epoch?: number): void {
    epoch ??= this.advanceEpoch();
    if (epoch !== this.visibilityEpoch) return;
    this.suspended = true;
    this.cancelScheduledRead();
    this.pending = undefined;
    this.geometryLane.clear();
    this.requestVisibility(epoch, false, Date.now());
    for (const element of this.elements) {
      if (element.hasAttribute(ELEMENT_STATUS)) element.removeAttribute(ELEMENT_STATUS);
    }
    this.lastSent = "";
    this.status("idle");
  }

  resume(): void {
    const epoch = this.advanceEpoch();
    this.failed = false;
    this.suspended = false;
    this.lastSent = "";
    this.discoverDirty = true;
    this.plans = new WeakMap();
    this.requestVisibility(epoch, true, Date.now());
    this.schedule();
  }

  private ensureSession(): Promise<number> {
    if (this.session !== undefined) return Promise.resolve(this.session);
    return this.sessionRequest ??= invoke<number>("begin_native_blur_session").then((session) => {
      this.session = session;
      return session;
    });
  }

  private advanceEpoch(): number {
    const epoch = ++this.visibilityEpoch;
    this.geometryLane.advance(epoch);
    this.visibilityLane.advance(epoch);
    this.pending = undefined;
    this.motion = undefined;
    return epoch;
  }

  private requestVisibility(
    epoch: number,
    visible: boolean,
    deadlineMs: number,
    closeAnimation?: NativeBlurCloseAnimation,
  ): void {
    // Do not allocate a native session/host for themes that never used native blur.
    const request = this.sessionRequest ?? (this.session === undefined ? undefined : Promise.resolve(this.session));
    if (!request) return;
    this.visibilityLane.offer(epoch, { request, epoch, visible, deadlineMs, closeAnimation });
  }

  private async dispatchVisibility(control: VisibilityControl): Promise<void> {
    const { request, epoch, visible, deadlineMs, closeAnimation } = control;
    const session = await request;
    if (epoch !== this.visibilityEpoch) return;
    const armed = await invoke<boolean>("set_native_blur_visibility", {
      session,
      epoch,
      visible,
      deadlineMs,
      closeAnimation,
    });
    if (armed && epoch === this.visibilityEpoch && session === this.session) {
      this.suspended = true;
      this.cancelScheduledRead();
      this.pending = undefined;
      this.geometryLane.clear();
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
      this.requestVisibility(this.advanceEpoch(), false, Date.now());
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
      let motion: NativeBlurCloseAnimation | undefined;
      let animateClose = false;
      if (!document.hidden && !this.suspended) {
        for (const element of this.elements) {
          const amount = Number.parseFloat(readStyle(element).getPropertyValue(AMOUNT) || "18");
          if (!Number.isFinite(amount) || amount <= 0) continue;
          const geometry = readNativeBlurGeometry(element, readStyle);
          if (!geometry) continue;
          animateClose ||= usesAnimatedClose(readStyle(element));
          if (!this.ids.has(element)) this.ids.set(element, this.nextId++);
          let cached = this.plans.get(element);
          const batchesMotion = element.hasAttribute("data-slu-native-blur-motion");
          const candidate = readOpacityPlan(element, cached?.plan, readStyle, batchesMotion);
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
          if (opacityAnimation && batchesMotion) {
            const animation = element.getAnimations().find((item) =>
              item.playState === "running" && typeof item.currentTime === "number" &&
              typeof item.startTime === "number" &&
              (item.effect as KeyframeEffect | null)?.target === element
            );
            if (animation && this.motion?.animation !== animation) {
              this.motion = { animation, armed: false };
              const finished = () => {
                if (this.motion?.animation !== animation) return;
                this.motion = undefined;
                this.schedule();
              };
              void animation.finished.then(finished, finished);
              try {
                const outlines = readNativeBlurMotion(element, animation, opacityAnimation.durationMs);
                if (outlines) {
                  this.motion.plan = {
                    regionId: this.ids.get(element)!,
                    scale: devicePixelRatio,
                    outlines,
                    opacity: opacityAnimation,
                  };
                }
              } catch { /* Fall back to live geometry for unsupported CSS. */ }
            }
            if (animation && this.motion?.animation === animation) motion = this.motion.plan;
          }
          if (regions.length === MAX_REGIONS) break;
        }
      }
      if (regions.length !== 1 || motion?.scale !== devicePixelRatio) motion = undefined;
      if (motion && this.motion?.armed) return;
      this.pending = { visibilityEpoch: this.visibilityEpoch, scale: devicePixelRatio, regions, animateClose, motion };
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

  private send(): void {
    const frame = this.pending;
    this.pending = undefined;
    if (!frame || this.failed || this.stopped || this.suspended) return;
    this.geometryLane.offer(frame.visibilityEpoch, frame);
  }

  private async dispatch(frame: Frame): Promise<void> {
    if (this.failed || this.stopped || this.suspended || frame.visibilityEpoch !== this.visibilityEpoch) return;
    if (frame.motion && this.motion?.plan === frame.motion && this.motion.armed) return;
    const key = JSON.stringify(frame);
    if (key === this.lastSent || (!frame.regions.length && this.session === undefined)) return;
    const session = await this.ensureSession();
    if (this.stopped || this.suspended || frame.visibilityEpoch !== this.visibilityEpoch) return;
    const sequence = ++this.sequence;
    const armed = await invoke<boolean>("sync_native_blur_regions", { session, sequence, ...frame });
    if (this.stopped || this.suspended || frame.visibilityEpoch !== this.visibilityEpoch || session !== this.session) {
      return;
    }
    if (frame.motion && this.motion?.plan === frame.motion) {
      if (armed) {
        this.motion.armed = true;
        this.cancelScheduledRead();
      } else {
        this.motion.plan = undefined;
      }
    }
    this.lastSent = key;
    const ids = new Set(frame.regions.map((region) => region.id));
    for (const element of this.elements) {
      if (ids.has(this.ids.get(element)!)) {
        if (element.getAttribute(ELEMENT_STATUS) !== "ready") element.setAttribute(ELEMENT_STATUS, "ready");
      } else if (element.hasAttribute(ELEMENT_STATUS)) element.removeAttribute(ELEMENT_STATUS);
    }
    this.status(frame.regions.length ? "ready" : "idle");
  }

  private status(value: string): void {
    if (this.stopped || document.documentElement.getAttribute(STATUS) === value) return;
    document.documentElement.setAttribute(STATUS, value);
    this.schedule();
  }

  private fail(error: unknown): void {
    if (this.failed || this.stopped) return;
    this.failed = true;
    this.geometryLane.clear();
    this.pending = undefined;
    this.lastSent = "";
    for (const element of this.elements) element.removeAttribute(ELEMENT_STATUS);
    if (this.session !== undefined) {
      this.requestVisibility(this.advanceEpoch(), false, Date.now());
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
    this.geometryLane.clear();
    this.requestVisibility(this.advanceEpoch(), false, Date.now());
  };
}
