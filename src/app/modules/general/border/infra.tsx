import { SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { ColorPicker, InputNumber, Switch } from 'antd';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { BorderSelectors } from '../../shared/app/selectors';
import { validateHexColor } from '../../shared/app/utils';
import { BorderActions } from './app';

export const BorderSettings = () => {
  const enabled = useAppSelector(BorderSelectors.enable);
  const offset = useAppSelector(BorderSelectors.offset);
  const width = useAppSelector(BorderSelectors.width);
  const color = useAppSelector(BorderSelectors.color);

  const dispatch = useAppDispatch();

  const toggleEnabled = useDispatchCallback(() => {
    dispatch(BorderActions.toggleEnable());
  });

  const updateOffset = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.updateOffset(value));
  });

  const updateWidth = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.updateWidth(value));
  });

  const updateColor = useDispatchCallback((color: any, hexcolor) => {
    dispatch(BorderActions.updateColor(validateHexColor(hexcolor)));
  });

  return (
    <SettingsSubGroup
      label={
        <SettingsOption>
          <span>Enable border on focus/activation</span>
          <Switch value={enabled} onChange={toggleEnabled} />
        </SettingsOption>
      }
    >
      <SettingsOption>
        <span>Border offset</span>
        <InputNumber value={offset} onChange={updateOffset} disabled={!enabled} />
      </SettingsOption>
      <SettingsOption>
        <span>Border width</span>
        <InputNumber value={width} onChange={updateWidth} disabled={!enabled} />
      </SettingsOption>
      <SettingsOption>
        <span>Border color</span>
        <ColorPicker
          disabledAlpha
          showText
          value={color}
          onChange={updateColor}
          disabled={!enabled}
        />
      </SettingsOption>
    </SettingsSubGroup>
  );
};
