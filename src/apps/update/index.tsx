import { UpdateModal } from './update';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { check, Update } from '@tauri-apps/plugin-updater';
import { ConfigProvider, theme } from 'antd';
import { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

const RouletteNode = document.getElementById('root');

const root = createRoot(RouletteNode || document.body);

function App() {
  const [update, setUpdate] = useState<Update | null>(null);

  useEffect(() => {
    check({})
      .then((update) => {
        if (!update) {
          getCurrent().close();
          return;
        }
        getCurrent().show();
        setUpdate(update);
      })
      .catch((e) => {
        console.error(e);
        getCurrent().close();
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

root.render(<App/>);