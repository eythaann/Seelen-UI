import { IconName } from '@icons';
import { invoke, SeelenCommand, Widget } from '@seelen-ui/lib';
import { SluPopupConfig, SluPopupContent as ISluPopupContent } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';

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
            getCurrentWindow().emit(`${entry.onClick}`);
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

const currentWidget = await Widget.getCurrentAsync();

export function App() {
  const [state, setState] = useState<SluPopupConfig>({
    title: [],
    content: [],
    footer: [],
  });

  useEffect(() => {
    invoke(SeelenCommand.GetPopupConfig, { instanceId: currentWidget.decoded.instanceId! })
      .then((data) => {
        setState(data);
        getCurrentWindow().show();
      })
      .catch((e) => {
        console.error(e);
        onCancel();
      });
  }, []);

  function onCancel() {
    getCurrentWindow().close();
  }

  return (
    <div className="popup">
      <header data-tauri-drag-region className="header">
        <div className="header-content">
          {state.title.map((subEntry, idx) => (
            <SluPopupContent key={idx} entry={subEntry} />
          ))}
        </div>
        <button className="header-close" onClick={onCancel}>
          <Icon iconName="CgClose" />
        </button>
      </header>

      <main className="content">
        {state.content.map((subEntry, idx) => (
          <SluPopupContent key={idx} entry={subEntry} />
        ))}
      </main>

      <footer className="footer">
        {state.footer.map((subEntry, idx) => (
          <SluPopupContent key={idx} entry={subEntry} />
        ))}
      </footer>
    </div>
  );
}
