import { SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { ColorPicker, InputNumber, Switch } from 'antd';
import { Color } from 'antd/es/color-picker';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { BorderSelectors } from '../../shared/app/selectors';
import { BorderActions } from './app';

import { HexColor } from '../../shared/domain/interfaces';

export const BorderSettings = () => {
  const enabled = useAppSelector(BorderSelectors.enable);
  const offset = useAppSelector(BorderSelectors.offset);
  const width = useAppSelector(BorderSelectors.width);
  const colorSingle = useAppSelector(BorderSelectors.colorSingle);
  const colorMonocle = useAppSelector(BorderSelectors.colorMonocle);
  const colorStack = useAppSelector(BorderSelectors.colorStack);

  const dispatch = useAppDispatch();

  const toggleEnabled = useDispatchCallback((value: boolean) => {
    dispatch(BorderActions.setEnable(value));
  });

  const updateOffset = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setOffset(value || 0));
  });

  const updateWidth = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setWidth(value || 0));
  });

  const updateColorSingle = useDispatchCallback((color: Color) => {
    dispatch(BorderActions.setColorSingle(color.toHexString() as HexColor));
  });

  const updateColorMonocle = useDispatchCallback((color: Color) => {
    dispatch(BorderActions.setColorMonocle(color.toHexString() as HexColor));
  });

  const updateColorStack = useDispatchCallback((color: Color) => {
    dispatch(BorderActions.setColorStack(color.toHexString() as HexColor));
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
        <span>Border color single</span>
        <ColorPicker
          disabledAlpha
          showText
          value={colorSingle}
          onChangeComplete={updateColorSingle}
          disabled={!enabled}
        />
      </SettingsOption>
      <SettingsOption>
        <span>Border color monocle</span>
        <ColorPicker
          disabledAlpha
          showText
          value={colorMonocle}
          onChangeComplete={updateColorMonocle}
          disabled={!enabled}
        />
      </SettingsOption>
      <SettingsOption>
        <span>Border color stack</span>
        <ColorPicker
          disabledAlpha
          showText
          value={colorStack}
          onChangeComplete={updateColorStack}
          disabled={!enabled}
        />
      </SettingsOption>
    </SettingsSubGroup>
  );
};
