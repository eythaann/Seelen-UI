import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { check, Update } from '@tauri-apps/plugin-updater';
import { ConfigProvider, theme } from 'antd';
import { useEffect, useState } from 'react';

import { UserSettingsLoader } from '../settings/modules/shared/store/storeApi';
import { useDarkMode } from '../shared/styles';
import { UpdateModal } from './update';

export function App() {
  const [update, setUpdate] = useState<Update | null>(null);

  const isDarkMode = useDarkMode();

  useEffect(() => {
    async function checkUpdate() {
      const webview = getCurrentWebviewWindow();
      const update = await check({});
      const { jsonSettings } = await new UserSettingsLoader().onlySettings().load();

      if (!update || (!jsonSettings.betaChannel && update.version.includes('beta'))) {
        webview.close();
        return;
      }

      webview.show();
      setUpdate(update);
    }
    checkUpdate().catch(() => getCurrentWebviewWindow().close());
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
