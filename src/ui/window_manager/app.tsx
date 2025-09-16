import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";

import { Layout } from "./modules/layout/infra";

import { ErrorBoundary } from "../weg/components/Error";

export function App() {
  useEffect(() => {
    let view = getCurrentWebviewWindow();
    view.show();
    view.emitTo(view.label, "complete-setup");
  }, []);

  return (
    <ErrorBoundary fallback={<div>Something went wrong on Window Manager</div>}>
      <Layout />
    </ErrorBoundary>
  );
}
