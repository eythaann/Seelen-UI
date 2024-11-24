import { Button } from 'antd';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { newSelectors } from '../shared/store/app/reducer';

import { SettingsGroup } from '../../components/SettingsBox';
import cs from './index.module.css';

export function ModsManager() {
  const plugins = useSelector(newSelectors.plugins);
  const widgets = [];

  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <div className={cs.title}>
          {t('mods.plugins')}: {plugins.length}
        </div>
        {plugins.map((plugin) => (
          <div key={plugin.id} className={cs.item}>
            <div className={cs.left}>
              <div className={cs.label}>{plugin.id}</div>
              <div>
                <b>{t('mods.target')}</b>: {plugin.target}
              </div>
            </div>
            <div className={cs.right}>
              <Button danger type="dashed" disabled={plugin.bundled}>
                {t('remove')}
              </Button>
            </div>
          </div>
        ))}
      </SettingsGroup>
      <SettingsGroup>
        <div className={cs.title}>
          {t('mods.widgets')}: {widgets.length}
        </div>
      </SettingsGroup>
    </>
  );
}
