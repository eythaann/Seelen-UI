import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { ColorPicker, InputNumber, Select } from 'antd';
import { Color } from 'antd/es/color-picker';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/app/hooks';
import { ContainerTopBarSelectors } from '../../shared/app/selectors';
import { OptionsFromEnum } from '../../shared/app/utils';
import { ContainerTopBarActions } from './app';

import { HexColor } from '../../shared/domain/interfaces';
import { ContainerTopBarMode } from './domain';

export const ContainerTopBarSettings = () => {
  const mode = useAppSelector(ContainerTopBarSelectors.mode);
  const height = useAppSelector(ContainerTopBarSelectors.height);
  const tabs = useAppSelector(ContainerTopBarSelectors.tabs);

  const enabled = mode !== ContainerTopBarMode.NEVER;

  const dispatch = useDispatch();

  const updateTopBarMode = (value: ContainerTopBarMode) => {
    dispatch(ContainerTopBarActions.setMode(value));
  };

  const onChangeHeight = (value: number | null) => {
    dispatch(ContainerTopBarActions.setHeight(value || 0));
  };

  const onChangeTabsWidth = (value: number | null) => {
    dispatch(
      ContainerTopBarActions.setTabs({
        ...tabs,
        width: value || 0,
      }),
    );
  };

  const onChangeTabsColor = (value: Color) => {
    dispatch(
      ContainerTopBarActions.setTabs({
        ...tabs,
        color: value.toHex() as HexColor,
      }),
    );
  };

  const onChangeTabsBackground = (value: Color) => {
    dispatch(
      ContainerTopBarActions.setTabs({
        ...tabs,
        background: value.toHex() as HexColor,
      }),
    );
  };

  return (
    <SettingsGroup>
      <SettingsSubGroup
        label={
          <SettingsOption>
            <span>Container Top Bar - Tabs</span>
            <Select value={mode} options={OptionsFromEnum(ContainerTopBarMode)} onChange={updateTopBarMode} />
          </SettingsOption>
        }
      >
        <SettingsOption>
          <span>Height</span>
          <InputNumber value={height} disabled={!enabled} onChange={onChangeHeight} />
        </SettingsOption>
        <SettingsOption>
          <span>Width</span>
          <InputNumber value={tabs.width} disabled={!enabled} onChange={onChangeTabsWidth} />
        </SettingsOption>
        <SettingsOption>
          <span>Text color</span>
          <ColorPicker
            disabledAlpha
            showText
            value={tabs.color}
            disabled={!enabled}
            onChangeComplete={onChangeTabsColor}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Background color</span>
          <ColorPicker
            disabledAlpha
            showText
            value={tabs.background}
            disabled={!enabled}
            onChangeComplete={onChangeTabsBackground}
          />
        </SettingsOption>
      </SettingsSubGroup>
    </SettingsGroup>
  );
};
