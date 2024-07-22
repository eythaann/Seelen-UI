import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { InputNumber, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { FancyToolbarActions } from './app';

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);
  const placeholders = useSelector(newSelectors.availablePlaceholders);
  const selectedStructure = useSelector(newSelectors.fancyToolbar.placeholder);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    dispatch(FancyToolbarActions.setEnabled(value));
  };

  const onSelectStructure = (value: string) => {
    dispatch(FancyToolbarActions.setPlaceholder(value));
  };

  const usingStructure = placeholders.find(
    (placeholder) => placeholder.info.filename === selectedStructure,
  );

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('toolbar.enable')}</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('toolbar.placeholder.select')}: </b>
          </div>
          <Select
            style={{ width: '200px' }}
            value={selectedStructure}
            options={placeholders.map((placeholder, idx) => ({
              key: `placeholder-${idx}`,
              label: placeholder.info.displayName,
              value: placeholder.info.filename,
            }))}
            onSelect={onSelectStructure}
          />
        </SettingsOption>
        <div>
          <p>
            <b>{t('toolbar.placeholder.author')}: </b>
            {usingStructure?.info.author}
          </p>
          <p>
            <b>{t('toolbar.placeholder.description')}: </b>
            {usingStructure?.info.description}
          </p>
        </div>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('toolbar.height')}</b>
          </div>
          <InputNumber
            value={settings.height}
            onChange={(value) => dispatch(FancyToolbarActions.setHeight(value || 0))}
            min={0}
          />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
