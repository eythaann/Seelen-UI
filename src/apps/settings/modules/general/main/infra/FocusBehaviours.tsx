import { SettingsOption } from '../../../../components/SettingsBox';
import { Select, Switch } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';
import { GeneralSettingsActions } from '../app';

import { FocusFollowsMouse } from '../domain';

export const FocusBehaviours = () => {
  const focusFollowsMouse = useAppSelector(GeneralSettingsSelectors.focusFollowsMouse);
  const mouseFollowFocus = useAppSelector(GeneralSettingsSelectors.mouseFollowFocus);

  const dispatch = useDispatch();

  const onChangeFocusFollowMouse = (value: FocusFollowsMouse) => {
    dispatch(GeneralSettingsActions.setFocusFollowsMouse(value));
  };

  const onChangeMouseFollowFocus = (value: boolean) => {
    dispatch(GeneralSettingsActions.setMouseFollowFocus(value));
  };

  return <div>
    <SettingsOption>
      <span>Mouse follows focus</span>
      <Switch value={mouseFollowFocus} onChange={onChangeMouseFollowFocus} />
    </SettingsOption>
    <SettingsOption>
      <span>Focus follows mouse mode</span>
      <Select<FocusFollowsMouse>
        value={focusFollowsMouse}
        allowClear
        placeholder="None"
        options={OptionsFromEnum(FocusFollowsMouse)}
        onChange={onChangeFocusFollowMouse}
      />
    </SettingsOption>
  </div>;
};