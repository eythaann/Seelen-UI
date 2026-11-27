import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Frame, Point, Rect, Size } from "@seelen-ui/types";
import { invoke, SeelenCommand } from "../../../handlers/mod.ts";

import { Widget_0 } from "./0_core.ts";

type unsubscriber = () => void;

let _sizeSubs: Set<(size: Size) => void>;
function registerSizeChange(cb: (size: Size) => void): unsubscriber {
  if (!_sizeSubs) {
    _sizeSubs = new Set();
    getCurrentWindow().onResized(({ payload }) => {
      _sizeSubs.forEach((cb) => cb(payload));
    });
  }
  _sizeSubs.add(cb);
  return () => {
    _sizeSubs.delete(cb);
  };
}

let _posSubs: Set<(size: Point) => void>;
function registerPosChange(cb: (size: Point) => void): unsubscriber {
  if (!_posSubs) {
    _posSubs = new Set();
    getCurrentWindow().onMoved(({ payload }) => {
      _posSubs.forEach((cb) => cb(payload));
    });
  }
  _posSubs.add(cb);
  return () => {
    _posSubs.delete(cb);
  };
}

export abstract class Widget_1 extends Widget_0 {
  private _size: Size = { width: 0, height: 0 };
  private _position: Point = { x: 0, y: 0 };

  override async prepare(): Promise<void> {
    await super.prepare();
    const win = getCurrentWindow();

    registerSizeChange((v) => (this._size = v));
    registerPosChange((v) => (this._position = v));

    this._size = await win.outerSize();
    this._position = await win.outerPosition();
  }

  get size(): Size {
    return { ...this._size };
  }

  get position(): Point {
    return { ...this._position };
  }

  get frame(): Frame {
    return { ...this._position, ...this._size };
  }

  get rect(): Rect {
    return {
      left: this._position.x,
      top: this._position.y,
      right: this._position.x + this._size.width,
      bottom: this._position.y + this._size.height,
    };
  }

  onResized(cb: (size: Size) => void): unsubscriber {
    return registerSizeChange(cb);
  }

  onMoved(cb: (pos: Point) => void): unsubscriber {
    return registerPosChange(cb);
  }

  onFrameChange(cb: (frame: Frame) => void): unsubscriber {
    const unsub1 = registerSizeChange((size) => cb({ ...size, ...this._position }));
    const unsub2 = registerPosChange((point) => cb({ ...point, ...this._size }));
    return () => {
      unsub1();
      unsub2();
    };
  }

  onRectChange(cb: (rect: Rect) => void): unsubscriber {
    return this.onFrameChange((frame) =>
      cb({
        left: frame.x,
        top: frame.y,
        right: frame.x + frame.width,
        bottom: frame.y + frame.height,
      })
    );
  }

  protected async __unsafe_setSelfPosition(rect: Rect, ref: Frame): Promise<void> {
    await invoke(SeelenCommand.SetSelfPosition, {
      rect: {
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        right: Math.round(rect.right),
        bottom: Math.round(rect.bottom),
      },
    });

    // optimistically update state, as arrived event after change is async
    ref.x = rect.left;
    ref.y = rect.top;
    ref.width = rect.right - rect.left;
    ref.height = rect.bottom - rect.top;
  }
}
