import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { Icon } from '@shared/components/Icon';
import { path } from '@tauri-apps/api';
import { Button } from 'antd';
import { useTranslation } from 'react-i18next';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';

export function SoundPacksView() {
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
              invoke(SeelenCommand.OpenFile, { path: await path.join(dataDir, 'sounds') });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
