import { SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { InputNumber, Switch } from 'antd';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { BorderSelectors } from '../../shared/app/selectors';
import { BorderActions } from './app';

export const BorderSettings = () => {
  const enabled = useAppSelector(BorderSelectors.enabled);
  const offset = useAppSelector(BorderSelectors.offset);
  const width = useAppSelector(BorderSelectors.width);

  const dispatch = useAppDispatch();

  const toggleEnabled = useDispatchCallback((value: boolean) => {
    dispatch(BorderActions.setEnabled(value));
  });

  const updateOffset = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setOffset(value || 0));
  });

  const updateWidth = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setWidth(value || 0));
  });

  return (
    <SettingsSubGroup
      label={
        <SettingsOption>
          <span>Enable windows border</span>
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
    </SettingsSubGroup>
  );
};
