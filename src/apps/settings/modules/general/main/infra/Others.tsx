import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { InputNumber } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { GeneralSettingsActions } from '../app';

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(GeneralSettingsSelectors.resizeDelta);

  const dispatch = useDispatch();

  const onChangeResizeDelta = (value: number | null) => {
    dispatch(GeneralSettingsActions.setResizeDelta(value || 0));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span>Resize delta (%)</span>
          <InputNumber value={resizeDelta} onChange={onChangeResizeDelta} min={1} max={40} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};
