import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { InputNumber, Switch } from 'antd';
import { useSelector } from 'react-redux';

import { AnimationsSelectors } from '../../shared/app/selectors';

export const AnimationsSettings = () => {
  const delay = useSelector(AnimationsSelectors.nativeDelay);
  const finishMinimization = useSelector(AnimationsSelectors.finishMiminization);

  return <SettingsGroup>
    <SettingsOption>
      <span>Wait minimization before restore workspace</span>
      <Switch value={finishMinimization} size="small" />
    </SettingsOption>
    <SettingsOption>
      <span>Native windows animations delay (ms)</span>
      <InputNumber value={delay} size="small" />
    </SettingsOption>
  </SettingsGroup>;
};