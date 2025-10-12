import { autoSizeWebviewBasedOnContent } from "@shared/AutoSizing";
import { Widget } from "@seelen-ui/lib";
import { useEffect } from "preact/hooks";
import { Bar } from "./app/bar";

const widget = Widget.getCurrent();

export function App() {
  useEffect(() => {
    autoSizeWebviewBasedOnContent({
      onResize: () => {
        widget.webview.center();
      },
    });
  }, []);

  return <Bar />;
}
