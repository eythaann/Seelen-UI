import { FancyToolbarSide, HideMode } from '@seelen-ui/lib';
import { Button, InputNumber, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { OptionsFromEnum } from '../shared/utils/app';
import { FancyToolbarActions } from './app';
import { Icon } from 'src/apps/shared/components/Icon';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);
  const delayToShow = useSelector(newSelectors.fancyToolbar.delayToShow);
  const delayToHide = useSelector(newSelectors.fancyToolbar.delayToHide);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    dispatch(FancyToolbarActions.setEnabled(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('toolbar.enable')}</b>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
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
          <SettingsOption>
            <div>{t('toolbar.dock_side')}</div>
            <Button.Group style={{ width: '60px' }}>
              {Object.values(FancyToolbarSide).map((side) => (
                <Button
                  key={side}
                  type={side === settings.position ? 'primary' : 'default'}
                  onClick={() => dispatch(FancyToolbarActions.setPosition(side))}
                >
                  <Icon iconName={`CgToolbar${side}`} size={18} />
                </Button>
              ))}
            </Button.Group>
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
                options={OptionsFromEnum(t, HideMode, 'toolbar.hide_mode')}
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
