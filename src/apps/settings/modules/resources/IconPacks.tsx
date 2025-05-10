import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { Icon } from '@shared/components/Icon';
import { path } from '@tauri-apps/api';
import { Button, Switch, Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import cs from './infra.module.css';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { ResourceCard } from './common';

export function IconPacksView() {
  const _active = useSelector(RootSelectors.iconPacks);
  const allIconPacks = useSelector(RootSelectors.availableIconPacks);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function toggleIconPack(filename: string) {
    if (_active.includes(filename)) {
      dispatch(RootActions.setIconPacks(_active.filter((x) => x !== filename)));
    } else {
      dispatch(RootActions.setIconPacks([..._active, filename]));
    }
  }

  function onReorder(activeIconPacks: string[]) {
    dispatch(RootActions.setIconPacks(activeIconPacks));
  }

  const disabled = allIconPacks.filter((x) => !_active.includes(x.metadata.filename));
  const enabled = _active
    .map((x) => allIconPacks.find((y) => y.metadata.filename === x)!)
    .filter(Boolean);

  return (
    <div className={cs.list}>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('general.icon_pack.label')}</b>
          <Tooltip title={t('general.icon_pack.open_folder')}>
            <Button
              type="default"
              onClick={async () => {
                const dataDir = await path.appDataDir();
                invoke(SeelenCommand.OpenFile, { path: await path.join(dataDir, 'icons') });
              }}
            >
              <Icon iconName="PiFoldersDuotone" />
            </Button>
          </Tooltip>
        </SettingsOption>
      </SettingsGroup>

      <b>Enabled</b>
      <Reorder.Group values={_active} onReorder={onReorder} className={cs.reorderGroup}>
        {enabled.map((iconPack) => (
          <Reorder.Item key={iconPack.id} value={iconPack.metadata.filename}>
            <ResourceCard
              resource={iconPack}
              kind="IconPack"
              actions={
                iconPack.id === '@system/icon-pack' ? undefined : (
                  <Switch
                    defaultChecked={true}
                    onChange={() => toggleIconPack(iconPack.metadata.filename)}
                  />
                )
              }
            />
          </Reorder.Item>
        ))}
      </Reorder.Group>

      <b>Disabled</b>
      {disabled.map((iconPack) => (
        <ResourceCard
          key={iconPack.id}
          resource={iconPack}
          kind="IconPack"
          actions={
            <Switch
              defaultChecked={false}
              onChange={() => toggleIconPack(iconPack.metadata.filename)}
            />
          }
        />
      ))}
    </div>
  );
}
