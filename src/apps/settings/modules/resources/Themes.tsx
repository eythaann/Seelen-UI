import { SeelenCommand } from '@seelen-ui/lib';
import { Icon } from '@shared/components/Icon';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Button, Switch, Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import cs from './infra.module.css';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { ResourceCard } from './common';

export function ThemesView() {
  const _active = useSelector(RootSelectors.selectedThemes);
  const allThemes = useSelector(RootSelectors.availableThemes);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function toggleTheme(theme: string) {
    if (_active.includes(theme)) {
      dispatch(RootActions.setSelectedThemes(_active.filter((x) => x !== theme)));
    } else {
      dispatch(RootActions.setSelectedThemes([..._active, theme]));
    }
  }

  function onReorder(themes: string[]) {
    dispatch(RootActions.setSelectedThemes(themes));
  }

  const disabled = allThemes.filter((x) => !_active.includes(x.metadata.filename));
  const enabled = _active
    .map((x) => allThemes.find((y) => y.metadata.filename === x)!)
    .filter(Boolean);

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('general.theme.label')}</b>
          <Tooltip title={t('general.theme.open_folder')}>
            <Button
              type="default"
              onClick={async () => {
                const dataDir = await path.appDataDir();
                invoke(SeelenCommand.OpenFile, { path: await path.join(dataDir, 'themes') });
              }}
            >
              <Icon iconName="PiFoldersDuotone" />
            </Button>
          </Tooltip>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        <b>Enabled</b>
        <Reorder.Group values={_active} onReorder={onReorder} className={cs.reorderGroup}>
          {enabled.map((theme) => (
            <Reorder.Item key={theme.id} value={theme.metadata.filename}>
              <ResourceCard
                resource={theme}
                kind="Theme"
                actions={
                  theme.id === '@default/theme' ? undefined : (
                    <Switch
                      defaultChecked={true}
                      onChange={() => toggleTheme(theme.metadata.filename)}
                    />
                  )
                }
              />
            </Reorder.Item>
          ))}
        </Reorder.Group>

        <b>Disabled</b>
        {disabled.map((theme) => (
          <ResourceCard
            key={theme.id}
            resource={theme}
            kind="Theme"
            actions={
              <Switch
                defaultChecked={false}
                onChange={() => toggleTheme(theme.metadata.filename)}
              />
            }
          />
        ))}
      </div>
    </>
  );
}
