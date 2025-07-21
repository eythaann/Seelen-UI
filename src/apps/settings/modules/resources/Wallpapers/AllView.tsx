import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { Icon } from '@shared/components/Icon';
import { path } from '@tauri-apps/api';
import { Button } from 'antd';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { NavLink } from 'react-router';

import cs from '../infra.module.css';

import { newSelectors } from '../../shared/store/app/reducer';

import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { ResourceCard } from '../common';

export function AllWallpapersView() {
  const wallpapers = useSelector(newSelectors.wallpapers);

  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('resources.open_folder')}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, { path: await path.join(dataDir, 'wallpapers') });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <b>{t('resources.import_wallpapers')}</b>
          <Button
            type="default"
            onClick={() => {
              invoke(SeelenCommand.StateRequestWallpaperAddition);
            }}
          >
            <Icon iconName="MdLibraryAdd" />
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        {wallpapers.map((resource) => (
          <ResourceCard
            key={resource.id}
            resource={resource}
            kind="Wallpaper"
            actions={
              <>
                <NavLink to={`/wallpaper/${resource.id.replace('@', '')}`}>
                  <Button type="text">
                    <Icon iconName="RiSettings4Fill" />
                  </Button>
                </NavLink>
              </>
            }
          />
        ))}
      </div>
    </>
  );
}
