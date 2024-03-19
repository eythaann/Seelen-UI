import { UpdateModal } from './update';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { check, Update } from '@tauri-apps/plugin-updater';
import { ConfigProvider, theme } from 'antd';
import { useEffect, useState } from 'react';

export function App() {
  const [update, setUpdate] = useState<Update | null>(null);

  useEffect(() => {
    const webview = getCurrent();
    check({})
      .then((update) => {
        if (!update) {
          webview.close();
          return;
        }
        webview.show();
        setUpdate(update);
      })
      .catch((e) => {
        console.error(e);
        webview.close();
      });
  }, []);

  if (!update) {
    return null;
  }

  return <ConfigProvider
    theme={{
      algorithm: window.matchMedia('(prefers-color-scheme: dark)').matches
        ? theme.darkAlgorithm
        : theme.defaultAlgorithm,
    }}
  >
    <UpdateModal update={update}/>
  </ConfigProvider>;
};
