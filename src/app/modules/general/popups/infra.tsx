import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { ColorPicker, InputNumber, Switch } from 'antd';
import { Color } from 'antd/es/color-picker';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../shared/app/selectors';
import { PopupActions } from './app';

export const PopupsSettings = () => {
  const popups = useAppSelector(GeneralSettingsSelectors.popups);

  const dispatch = useAppDispatch();

  const onChangeX = (v: number | null) => dispatch(PopupActions.setX(v));
  const onChangeY = (v: number | null) => dispatch(PopupActions.setY(v));
  const onChangeWidth = (v: number | null) => dispatch(PopupActions.setWidth(v || 0));
  const onChangeHeight = (v: number | null) => dispatch(PopupActions.setHeight(v || 0));
  const onChangeBorderWidth = (v: number | null) => dispatch(PopupActions.setBorderWidth(v || 0));

  const updateTextColor = useDispatchCallback((color: Color) => {
    dispatch(PopupActions.setTextColor(color.toHexString()));
  });

  const updateBKColor = useDispatchCallback((color: Color) => {
    dispatch(PopupActions.setBackground(color.toHexString()));
  });

  const updateBorderColor = useDispatchCallback((color: Color) => {
    dispatch(PopupActions.setBorderColor(color.toHexString()));
  });

  return (
    <>
      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption
              label="Popups"
              trigger={
                <Switch
                  value={popups.enable}
                  onChange={(v) => dispatch(PopupActions.setEnable(v))}
                />
              }
            />
          }
        >
          <SettingsOption
            label="Position X"
            trigger={<InputNumber placeholder="center" value={popups.x} onChange={onChangeX} />}
          />
          <SettingsOption
            label="Position Y"
            trigger={<InputNumber placeholder="center" value={popups.y} onChange={onChangeY} />}
          />
          <SettingsOption
            label="Width"
            trigger={<InputNumber value={popups.width} onChange={onChangeWidth} />}
          />
          <SettingsOption
            label="Height"
            trigger={<InputNumber value={popups.height} onChange={onChangeHeight} />}
          />
          <SettingsOption
            label="Text color"
            trigger={
              <ColorPicker
                showText
                disabledAlpha
                value={popups.textColor}
                onChangeComplete={updateTextColor}
              />
            }
          />
          <SettingsOption
            label="Background"
            trigger={
              <ColorPicker
                showText
                disabledAlpha
                value={popups.background}
                onChangeComplete={updateBKColor}
              />
            }
          />
          <SettingsOption
            label="Border width"
            trigger={<InputNumber value={popups.borderWidth} onChange={onChangeBorderWidth} />}
          />
          <SettingsOption
            label="Border color"
            trigger={
              <ColorPicker
                showText
                disabledAlpha
                value={popups.borderColor}
                onChangeComplete={updateBorderColor}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
};
