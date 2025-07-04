import { IconName } from '@icons';
import { signal } from '@preact/signals';
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from '@seelen-ui/lib';
import { SluPopupConfig, SluPopupContent as ISluPopupContent } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { emit } from '@tauri-apps/api/event';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/window';

const currentWidget = await Widget.getCurrentAsync();

const state = signal<SluPopupConfig>({
  width: 0,
  height: 0,
  title: [],
  content: [],
  footer: [],
});

invoke(SeelenCommand.GetPopupConfig, { instanceId: currentWidget.decoded.instanceId! })
  .then(async (data) => {
    state.value = data;
    currentWidget.webview.setTitle(getOnlyText(data.title));
    await currentWidget.webview.show();
    await currentWidget.webview.setFocus();
  })
  .catch((err) => {
    console.error(err);
    closePopup();
  });

subscribe(SeelenEvent.PopupContentChanged, async (e) => {
  const oldWidth = state.value.width;
  const oldHeight = state.value.height;

  const widthDiff = e.payload.width - oldWidth;
  const heightDiff = e.payload.height - oldHeight;

  const newX = window.screenLeft - widthDiff / 2;
  const newY = window.screenTop - heightDiff / 2;

  state.value = e.payload;
  currentWidget.webview.setTitle(getOnlyText(e.payload.title));
  currentWidget.webview.setPosition(new LogicalPosition(newX, newY));
  currentWidget.webview.setSize(new LogicalSize(e.payload.width, e.payload.height));
});

function closePopup() {
  currentWidget.webview.close();
}

export function App() {
  return (
    <div className="popup">
      <header data-tauri-drag-region className="header">
        <div className="header-content">
          {state.value.title.map((subEntry, idx) => (
            <SluPopupContent key={idx} entry={subEntry} />
          ))}
        </div>
        <button className="header-close" onClick={closePopup}>
          <Icon iconName="CgClose" />
        </button>
      </header>

      <main className="content">
        {state.value.content.map((subEntry, idx) => (
          <SluPopupContent key={idx} entry={subEntry} />
        ))}
      </main>

      <footer className="footer">
        {state.value.footer.map((subEntry, idx) => (
          <SluPopupContent key={idx} entry={subEntry} />
        ))}
      </footer>
    </div>
  );
}

function SluPopupContent({ entry }: { entry: ISluPopupContent }) {
  switch (entry.type) {
    case 'text':
      return (
        <p className="text" style={entry.styles || {}}>
          {entry.value}
        </p>
      );
    case 'icon':
      return <Icon className="icon" iconName={entry.name as IconName} style={entry.styles || {}} />;
    case 'image':
      return <img className="image" src={entry.href} style={entry.styles || {}} alt={entry.href} />;
    case 'button':
      return (
        <button
          className="button"
          onClick={() => {
            emit(`${entry.onClick}`);
          }}
          style={entry.styles || {}}
        >
          {entry.inner.map((subEntry, idx) => (
            <SluPopupContent key={idx} entry={subEntry} />
          ))}
        </button>
      );
    case 'group':
      return (
        <div className="group" style={entry.styles || {}}>
          {entry.items.map((subEntry, idx) => (
            <SluPopupContent key={idx} entry={subEntry} />
          ))}
        </div>
      );
    default:
      return null;
  }
}

function getOnlyText(content: ISluPopupContent[]) {
  let text = '';
  for (const entry of content) {
    if (entry.type === 'text') {
      text += `${entry.value} `;
    }

    if (entry.type === 'group') {
      text += getOnlyText(entry.items);
      text += ' ';
    }
  }
  return text;
}
