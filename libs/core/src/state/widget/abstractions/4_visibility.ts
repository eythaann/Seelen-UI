import { debounce } from "../../../utils/async.ts";
import { Widget_3 } from "./3_autosize.ts";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

  public async show(): Promise<void> {
    const token = ++this._visibilityToken;
    debouncedClose.cancel();
    await this.window.show();
    // a hide() was called while showing, it already owns the attribute.
    if (token === this._visibilityToken) {
      delete globalThis.document.documentElement.dataset.widgetHidden;
    }
  }

  public async hide(): Promise<void> {
    const token = ++this._visibilityToken;
    globalThis.document.documentElement.dataset.widgetHidden = "";

    await this.waitHideAnimations();
    if (token !== this._visibilityToken) {
      return;
    }
    await this.window.hide();
    // a show() may have been called while the window was hiding, don't schedule the close.
    if (this._destroyOnHide && token === this._visibilityToken) {
      debouncedClose();
    }
  }

  /**
   * Resolves once the finite animations/transitions triggered by the visibility change end.
   * Canceled animations (e.g. a transition reversed by a new show) also resolve it.
   */
  private async waitHideAnimations(): Promise<void> {
    // `getAnimations` forces a style recalc, so it already includes the ones just triggered
    const animations = globalThis.document
      .getAnimations()
      .filter((animation) => animation.effect?.getComputedTiming().iterations !== Infinity);
    if (animations.length === 0) {
      return;
    }

    let timeout: ReturnType<typeof setTimeout>;
    await Promise.race([
      Promise.allSettled(animations.map((animation) => animation.finished)),
      new Promise((resolve) => (timeout = setTimeout(resolve, HIDE_ANIMATION_MAX_MS))),
    ]);
    clearTimeout(timeout!);
  }
}
