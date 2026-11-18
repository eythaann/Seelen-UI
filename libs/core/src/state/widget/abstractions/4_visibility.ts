import { debounce } from "../../../utils/async.ts";
import { Widget_3 } from "./3_autosize.ts";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { beginNativeBlurClose, finishNativeBlurClose, resumeNativeBlur } from "../../theme/nativeBlur.ts";
import {
  captureHideAnimations,
  collectHideAnimations,
  hideAnimationBudget,
  waitHideAnimations,
} from "./hideAnimations.ts";

/** Max time given to widgets to play their hide animation before the window is really hidden */
const HIDE_ANIMATION_MAX_MS = 200;

const debouncedClose = debounce(async () => {
  const window = getCurrentWindow();
  // safety net, never close a window that the user is looking at
  if (!(await window.isVisible())) {
    await window.close();
  }
}, 30_000);

export class Widget_4 extends Widget_3 {
  protected _destroyOnHide = false;
  /**
   * Incremented on every show/hide call, so any in-flight call can detect it was superseded
   * by a newer one after each `await` and bail out.
   */
  private _visibilityToken = 0;
  private _hideOperation?: { token: number; promise: Promise<void> };

  protected _visible = false;

  /**
   * Cached visibility, updated synchronously on `show()`/`hide()` calls (a hide in progress counts as hidden).
   * Windows shown/hidden directly via `window.show()`/`window.hide()` are not tracked.
   */
  public get visible(): boolean {
    return this._visible;
  }

  private _visibilitySubs = new Set<(visible: boolean) => void>();
  /**
   * Called synchronously when `show()`/`hide()` change the cached visibility, before the hide
   * animation plays (so the widget can follow it).
   */
  public onVisibilityChange(cb: (visible: boolean) => void): () => void {
    this._visibilitySubs.add(cb);
    return () => {
      this._visibilitySubs.delete(cb);
    };
  }

  private setVisible(visible: boolean): void {
    if (this._visible === visible) return;
    this._visible = visible;
    this._visibilitySubs.forEach((cb) => cb(visible));
  }

  public async show(): Promise<void> {
    const token = ++this._visibilityToken;
    this.setVisible(true);
    debouncedClose.cancel();
    resumeNativeBlur();
    await this.window.show();
    // a hide() was called while showing, it already owns the attribute.
    if (token === this._visibilityToken) {
      delete globalThis.document.documentElement.dataset.widgetHidden;
    }
  }

  public hide(): Promise<void> {
    if (this._hideOperation?.token === this._visibilityToken) return this._hideOperation.promise;
    const token = ++this._visibilityToken;
    // Cached visibility flips immediately, the `widgetHidden` attribute is set by the native
    // blur close sequence right before the hide animation starts.
    this.setVisible(false);
    const promise = this.performHide(token).finally(() => {
      if (this._hideOperation?.token === token) this._hideOperation = undefined;
    });
    this._hideOperation = { token, promise };
    return promise;
  }

  private async performHide(token: number): Promise<void> {
    const before = captureHideAnimations(globalThis.document.getAnimations());
    // Native blur exits before the HTML close animation starts, without waiting for an ACK.
    const close = beginNativeBlurClose(HIDE_ANIMATION_MAX_MS, () => {
      globalThis.document.documentElement.dataset.widgetHidden = "";
    });
    const animations = collectHideAnimations(before, globalThis.document.getAnimations());
    const budget = hideAnimationBudget(animations, HIDE_ANIMATION_MAX_MS);
    // Normal completion follows animation.finished; the native deadline is only the watchdog.
    // Never use nominal CSS duration as a hard deadline: its timeline may lag wall-clock time.
    // IPC is never a prerequisite to HWND hide.
    await waitHideAnimations(animations, budget);
    if (token !== this._visibilityToken) {
      return;
    }
    const hidden = this.window.hide();
    finishNativeBlurClose(close);
    await hidden;
    // a show() may have been called while the window was hiding, don't schedule the close.
    if (this._destroyOnHide && token === this._visibilityToken) {
      debouncedClose();
    }
  }
}
