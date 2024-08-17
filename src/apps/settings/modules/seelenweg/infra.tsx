import { SeelenWegHideMode, SeelenWegMode, SeelenWegSide } from '../../../shared/schemas/Seelenweg';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { InputNumber, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';

import { useAppDispatch, useAppSelector } from '../shared/utils/infra';

import { RootSelectors } from '../shared/store/app/selectors';
import { OptionsFromEnum } from '../shared/utils/app';
import { SeelenWegActions } from './app';

export const SeelenWegSettings = () => {
  const settings = useAppSelector(RootSelectors.seelenweg);

  const dispatch = useAppDispatch();
  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    dispatch(SeelenWegActions.setEnabled(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('weg.enable')}</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('weg.label')}>
          <SettingsOption>
            <div>{t('weg.width')}</div>
            <Select
              style={{ width: '120px' }}
              value={settings.mode}
              options={OptionsFromEnum(SeelenWegMode)}
              onChange={(value) => dispatch(SeelenWegActions.setMode(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.auto_hide')}</div>
            <Select
              style={{ width: '120px' }}
              value={settings.hideMode}
              options={OptionsFromEnum(SeelenWegHideMode)}
              onChange={(value) => dispatch(SeelenWegActions.setHideMode(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.dock_side')}</div>
            <Select
              style={{ width: '120px' }}
              value={settings.position}
              options={OptionsFromEnum(SeelenWegSide)}
              onChange={(value) => dispatch(SeelenWegActions.setPosition(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.margin')}</div>
            <InputNumber
              value={settings.margin}
              onChange={(value) => dispatch(SeelenWegActions.setMargin(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.padding')}</div>
            <InputNumber
              value={settings.padding}
              onChange={(value) => dispatch(SeelenWegActions.setPadding(value || 0))}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('weg.items.label')}>
          <SettingsOption>
            <div>{t('weg.items.size')}</div>
            <InputNumber
              value={settings.size}
              onChange={(value) => dispatch(SeelenWegActions.setSize(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.items.zoom_size')}</div>
            <InputNumber
              value={settings.zoomSize}
              onChange={(value) => dispatch(SeelenWegActions.setZoomSize(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.items.gap')}</div>
            <InputNumber
              value={settings.spaceBetweenItems}
              onChange={(value) => dispatch(SeelenWegActions.setSpaceBetweenItems(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t('weg.items.visible_separators')}</div>
            <Switch checked={settings.visibleSeparators} onChange={(value) => dispatch(SeelenWegActions.setVisibleSeparators(value))} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
};
