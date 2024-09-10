import { Select, Switch } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { startup } from '../../../shared/tauri/infra';
import { useAppDispatch } from '../../../shared/utils/infra';

import { RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';

import { LanguageList } from '../../../../../shared/lang';
import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { Colors } from './Colors';
import { Themes } from './Themes';
import { Wallpaper } from './Wallpaper';

export function General() {
  const [changingAutostart, setChangingAutostart] = useState(false);

  const autostartStatus = useSelector(RootSelectors.autostart);
  const language = useSelector(RootSelectors.language);

  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const onAutoStart = async (value: boolean) => {
    setChangingAutostart(true);
    if (value) {
      await startup.enable();
    } else {
      await startup.disable();
    }
    setChangingAutostart(false);
    dispatch(RootActions.setAutostart(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>{t('general.startup')}</span>
          <Switch
            onChange={onAutoStart}
            value={!!autostartStatus}
            loading={changingAutostart || autostartStatus === null}
          />
        </SettingsOption>
        <SettingsOption>
          <b>{t('general.language')}:</b>
          <Select
            showSearch
            optionFilterProp="label"
            style={{ width: '200px' }}
            value={language}
            options={[...LanguageList]}
            onSelect={(value) => dispatch(RootActions.setLanguage(value))}
          />
        </SettingsOption>
      </SettingsGroup>

      <Colors />

      <SettingsGroup>
        <Wallpaper />
      </SettingsGroup>

      <SettingsGroup>
        <div style={{ marginBottom: '6px' }}>
          <b>{t('general.theme.label')}</b>
        </div>
        <Themes />
      </SettingsGroup>
    </>
  );
}
