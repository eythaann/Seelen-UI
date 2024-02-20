import { SettingsOption } from '../../../../components/SettingsBox';
import { Select, Switch } from 'antd';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';

import { FocusFollowsMouse } from '../domain';

export const FocusBehaviours = () => {
  const focusFollowsMouse = useAppSelector(GeneralSettingsSelectors.focusFollowsMouse);
  const mouseFollowFocus = useAppSelector(GeneralSettingsSelectors.mouseFollowFocus);

  return <div>
    <SettingsOption>
      <span>Mouse follows focus</span>
      <Switch value={mouseFollowFocus} />
    </SettingsOption>
    <SettingsOption>
      <span>Focus follows mouse mode</span>
      <Select
        value={focusFollowsMouse}
        options={OptionsFromEnum(FocusFollowsMouse)}
      />
    </SettingsOption>
  </div>;
};