import { getCurrentWebview, type Webview } from "@tauri-apps/api/webview";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { invoke, SeelenCommand } from "../../../../mod.ts";

export abstract class Widget_0 {
  private _windowId: string = "";

  /** current webview where the widget is running */
  readonly webview: Webview = getCurrentWebview();
  /** current window where the widget is running */
  readonly window: Window = getCurrentWindow();

  /** Returns the current window id of the widget */
  get windowId(): string {
    return this._windowId;
  }

  protected async prepare(): Promise<void> {
    this._windowId = await invoke(SeelenCommand.GetSelfWindowId);
  }
}
