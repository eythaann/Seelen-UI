import { SeelenWegMode, SeelenWegSide } from '../../../utils/interfaces/Weg';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { InputNumber, Select, Switch } from 'antd';

import { useAppDispatch, useAppSelector } from '../shared/utils/infra';
import { RootSelectors } from '../shared/store/app/selectors';
import { OptionsFromEnum } from '../shared/utils/app';
import { SeelenWegActions } from './app';

export const SeelenWegSettings = () => {
  const settings = useAppSelector(RootSelectors.seelenweg);

  const dispatch = useAppDispatch();

  const onToggleEnable = (value: boolean) => {
    dispatch(SeelenWegActions.setEnable(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Enable SeelenWeg</b>
          </div>
          <Switch checked={settings.enable} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Dock/Taskbar">
          <SettingsOption>
            <div>Width</div>
            <Select
              style={{ width: '120px' }}
              value={settings.mode}
              options={OptionsFromEnum(SeelenWegMode)}
              onChange={(value) => dispatch(SeelenWegActions.setMode(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>Side to show dock</div>
            <Select
              style={{ width: '120px' }}
              value={settings.position}
              options={OptionsFromEnum(SeelenWegSide)}
              onChange={(value) => dispatch(SeelenWegActions.setPosition(value))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>Margin</div>
            <InputNumber
              value={settings.margin}
              onChange={(value) => dispatch(SeelenWegActions.setMargin(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>Padding</div>
            <InputNumber
              value={settings.padding}
              onChange={(value) => dispatch(SeelenWegActions.setPadding(value || 0))}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Separators">
          <SettingsOption>
            <div>Visible</div>
            <Switch checked={settings.visibleSeparators} onChange={(value) => dispatch(SeelenWegActions.setVisibleSeparators(value))} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Items">
          <SettingsOption>
            <div>Size</div>
            <InputNumber
              value={settings.size}
              onChange={(value) => dispatch(SeelenWegActions.setSize(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>Size on hover</div>
            <InputNumber
              value={settings.zoomSize}
              onChange={(value) => dispatch(SeelenWegActions.setZoomSize(value || 0))}
            />
          </SettingsOption>
          <SettingsOption>
            <div>Space between items</div>
            <InputNumber
              value={settings.spaceBetweenItems}
              onChange={(value) => dispatch(SeelenWegActions.setSpaceBetweenItems(value || 0))}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
};
