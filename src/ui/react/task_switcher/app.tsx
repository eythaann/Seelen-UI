import { Widget } from "@seelen-ui/lib";
import { useEffect } from "preact/hooks";
import { Bar } from "./app/bar.tsx";

const widget = Widget.getCurrent();

export function App() {
  useEffect(() => {
    widget.webview.onResized(() => {
      widget.webview.center();
    });
    widget.autoSizeWebviewByElement(document.getElementById("root")!);
  }, []);

  return <Bar />;
}
