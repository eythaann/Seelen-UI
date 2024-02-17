import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { ColorPicker, InputNumber, Select } from 'antd';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { ContainerTopBarSelectors } from '../../shared/app/selectors';

import { ContainerTopBarMode } from './domain';

export const ContainerTopBarSettings = () => {
  const mode = useAppSelector(ContainerTopBarSelectors.mode);
  const height = useAppSelector(ContainerTopBarSelectors.height);
  const tabs = useAppSelector(ContainerTopBarSelectors.tabs);

  const enabled = mode !== ContainerTopBarMode.NEVER;

  const dispatch = useAppDispatch();

  return (
    <SettingsGroup>
      <SettingsSubGroup
        label={
          <SettingsOption>
            <span>Container Top Bar - Tabs</span>
            <Select
              value={mode}
              size="small"
              options={Object.values(ContainerTopBarMode).map((op) => ({
                label: op,
              }))}
            />
          </SettingsOption>
        }
      >
        <SettingsOption>
          <span>Height</span>
          <InputNumber size="small" value={height} disabled={!enabled} />
        </SettingsOption>
        <SettingsOption>
          <span>Width</span>
          <InputNumber size="small" value={tabs.width} disabled={!enabled} />
        </SettingsOption>
        <SettingsOption>
          <span>Text color</span>
          <ColorPicker size="small" disabledAlpha showText value={tabs.color} disabled={!enabled} />
        </SettingsOption>
        <SettingsOption>
          <span>Background color</span>
          <ColorPicker
            size="small"
            disabledAlpha
            showText
            value={tabs.background}
            disabled={!enabled}
          />
        </SettingsOption>
      </SettingsSubGroup>
    </SettingsGroup>
  );
};
