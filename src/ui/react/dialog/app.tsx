import { signal } from "@preact/signals";
import { Widget } from "@seelen-ui/lib";
import type { Dialog, DialogContent as IDialogContent } from "@seelen-ui/lib/types";
import { LogicalSize } from "@tauri-apps/api/window";
import { Icon } from "libs/ui/react/components/Icon";
import { useEffect } from "preact/hooks";

const currentWidget = Widget.getCurrent();
const state = signal<Dialog | null>(null);
let owner = "";

currentWidget.onTrigger(async ({ customArgs }) => {
  state.value = (customArgs?.dialog as Dialog) ?? null;
  owner = (customArgs?.owner as string) ?? "";

  if (!state.value) {
    currentWidget.hide();
    return;
  }

  Widget.self.window.setTitle(getOnlyText(state.value.title));
  await Widget.self.window.setSize(new LogicalSize(state.value.width, state.value.height));
  await Widget.self.window.center();
  await Widget.self.show();
  await Widget.self.focus();
});

export function App() {
  useEffect(() => {
    Widget.self.ready();
  }, []);

  const dialog = state.value;
  if (!dialog) return null;

  return (
    <div className="dialog">
      <header data-tauri-drag-region className="header">
        <div className="header-content">
          {dialog.title.map((entry, idx) => <SluPopupContent key={idx} entry={entry} />)}
        </div>
        <button className="header-close" onClick={() => currentWidget.window.close()}>
          <Icon iconName="CgClose" />
        </button>
      </header>

      <main className="content">
        {dialog.content.map((entry, idx) => <SluPopupContent key={idx} entry={entry} />)}
      </main>

      <footer className="footer">
        {dialog.footer.map((entry, idx) => <SluPopupContent key={idx} entry={entry} />)}
      </footer>
    </div>
  );
}

function SluPopupContent({ entry }: { entry: IDialogContent }) {
  switch (entry.type) {
    case "text":
      return (
        <p className="text" style={entry.styles || {}}>
          {entry.value}
        </p>
      );
    case "icon":
      return <Icon className="icon" iconName={entry.name as any} style={entry.styles || {}} />;
    case "image":
      return <img className="image" src={entry.href} style={entry.styles || {}} alt={entry.href} />;
    case "button":
      return (
        <button
          data-skin={entry.skin || "default"}
          onClick={() => handleButtonClick(entry.onClick)}
          style={entry.styles || {}}
        >
          {entry.inner.map((subEntry, idx) => <SluPopupContent key={idx} entry={subEntry} />)}
        </button>
      );
    case "group":
      return (
        <div className="group" style={entry.styles || {}}>
          {entry.items.map((subEntry, idx) => <SluPopupContent key={idx} entry={subEntry} />)}
        </div>
      );
    case "loader":
      return <div className="loader" style={entry.styles || {}} />;
    default:
      return null;
  }
}

function handleButtonClick(onClick: string) {
  if (onClick === "exit") {
    Widget.self.window.close();
    return;
  }

  if (owner) {
    // frontend-triggered: send event back to the owner widget
    currentWidget.webview.emitTo(owner, onClick, null);
  } else {
    // backend-triggered: emit globally so backend listeners can receive it
    currentWidget.webview.emit(onClick, null);
  }

  currentWidget.hide();
}

function getOnlyText(content: IDialogContent[]) {
  let text = "";
  for (const entry of content) {
    if (entry.type === "text") {
      text += `${entry.value} `;
    }

    if (entry.type === "group") {
      text += getOnlyText(entry.items);
      text += " ";
    }
  }
  return text;
}
