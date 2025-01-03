import { HideMode } from '@seelen-ui/lib';
import { InputNumber, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { OptionsFromEnum } from '../shared/utils/app';
import { FancyToolbarActions } from './app';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);
  const placeholders = useSelector(newSelectors.availablePlaceholders);
  const selectedStructure = useSelector(newSelectors.fancyToolbar.placeholder);
  const delayToShow = useSelector(newSelectors.fancyToolbar.delayToShow);
  const delayToHide = useSelector(newSelectors.fancyToolbar.delayToHide);

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
          <b>{t('toolbar.enable')}</b>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t('toolbar.placeholder.select')}: </b>
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
        <SettingsSubGroup label={t('toolbar.label')}>
          <SettingsOption>
            <b>{t('toolbar.height')}</b>
            <InputNumber
              value={settings.height}
              onChange={(value) => dispatch(FancyToolbarActions.setHeight(value || 0))}
              min={0}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <b>{t('toolbar.auto_hide')}</b>
              <Select
                style={{ width: '120px' }}
                value={settings.hideMode}
                options={OptionsFromEnum(HideMode, 'toolbar.hide_mode')}
                onChange={(value) => dispatch(FancyToolbarActions.setHideMode(value))}
              />
            </SettingsOption>
          }
        >
          <SettingsOption>
            <span>{t('toolbar.use_multiple_monitor_overlap_logic')}</span>
            <Switch
              disabled={settings.hideMode != HideMode.OnOverlap}
              checked={settings.useMultiMonitorOverlapLogic}
              onChange={(value) => dispatch(FancyToolbarActions.setUseMultiMonitorOverlapLogic(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <b>{t('toolbar.delay_to_show')} (ms)</b>
            <InputNumber
              value={delayToShow}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => {
                dispatch(FancyToolbarActions.setDelayToShow(value || 0));
              }}
            />
          </SettingsOption>
          <SettingsOption>
            <b>{t('toolbar.delay_to_hide')} (ms)</b>
            <InputNumber
              value={delayToHide}
              min={0}
              disabled={settings.hideMode === HideMode.Never}
              onChange={(value) => {
                dispatch(FancyToolbarActions.setDelayToHide(value || 0));
              }}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
}
