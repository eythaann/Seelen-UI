import { useDarkMode } from '../shared/styles';
import { UpdateModal } from './update';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { check, Update } from '@tauri-apps/plugin-updater';
import { ConfigProvider, theme } from 'antd';
import { useEffect, useState } from 'react';

export function App() {
  const [update, setUpdate] = useState<Update | null>(null);

  const isDarkMode = useDarkMode();

  useEffect(() => {
    const webview = getCurrentWebviewWindow();
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

  return (
    <ConfigProvider
      theme={{
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <UpdateModal update={update} />
    </ConfigProvider>
  );
}
