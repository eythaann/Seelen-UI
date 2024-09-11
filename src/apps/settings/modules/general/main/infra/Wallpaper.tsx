import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';
import { SeelenCommand } from 'seelen-core';

import { dialog } from '../../../shared/tauri/infra';

import { RootActions } from '../../../shared/store/app/reducer';

import { Monitor } from '../../../../components/monitor';
import { SettingsOption } from '../../../../components/SettingsBox';
import cs from './index.module.css';

export function Wallpaper() {
  const { t } = useTranslation();
  const dispatch = useDispatch();

  async function loadWallpaper() {
    const file = await dialog.open({
      title: t('general.wallpaper.select'),
      filters: [
        { name: 'images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'tif', 'tiff'] },
      ],
    });

    if (!file) {
      return;
    }

    await invoke(SeelenCommand.StateSetWallpaper, { path: file.path });
    dispatch(RootActions.setWallpaper(convertFileSrc(file.path)));
  }

  return (
    <>
      <SettingsOption>
        <Monitor />
        <div className={cs.wallpaperButton}>
          <Button onClick={loadWallpaper} size="middle">
            {t('general.wallpaper.select')}
          </Button>
        </div>
      </SettingsOption>
    </>
  );
}
