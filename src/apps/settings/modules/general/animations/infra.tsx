import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { InputNumber, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { AnimationsSelectors } from '../../shared/app/selectors';
import { AnimationsActions } from './app';

export const AnimationsSettings = () => {
  const delay = useSelector(AnimationsSelectors.nativeDelay);
  const finishMinimization = useSelector(AnimationsSelectors.finishMiminization);

  const dispatch = useDispatch();

  const onUpdateDelay = (value: number | null) => {
    dispatch(AnimationsActions.setNativeDelay(value || 0));
  };

  const onChangeFinishMinimization = (value: boolean) => {
    dispatch(AnimationsActions.setFinishMiminization(value));
  };

  return (
    <SettingsGroup>
      <SettingsOption>
        <span>Wait minimization before restore workspace</span>
        <Switch value={finishMinimization} onChange={onChangeFinishMinimization} />
      </SettingsOption>
      <SettingsOption>
        <span>Native windows animations delay (ms)</span>
        <InputNumber value={delay} onChange={onUpdateDelay} />
      </SettingsOption>
    </SettingsGroup>
  );
};
