import { VariableConvention } from '../../../utils/schemas';
import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Input, Switch, Tooltip } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { AhkVariablesActions, KeyCodeToAHK } from './app';

export function Shortcuts() {
  const ahkEnable = useSelector(RootSelectors.ahkEnabled);
  const ahkVariables = useSelector(RootSelectors.ahkVariables);

  const dispatch = useDispatch();

  const onChangeEnabled = (value: boolean) => {
    dispatch(RootActions.setAhkEnabled(value));
    dispatch(RootActions.setToBeSaved(true));
  };

  const onChangeVar = (name: string, e: React.KeyboardEvent<HTMLInputElement>) => {
    const result = KeyCodeToAHK(e);
    if (result) {
      dispatch(AhkVariablesActions.setVariable({ name, value: result }));
    }
  };

  return (
    <div>
      <SettingsGroup>
        <SettingsOption>
          <span>
            Enable Seelen UI shortcuts{' '}
            <Tooltip
              title="Disable if you will implement your own shortcuts using the CLI."
            >
              🛈
            </Tooltip>
          </span>
          <Switch value={ahkEnable} onChange={onChangeEnabled} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        {
          Object.entries(ahkVariables).map(([name, value]) => {
            let label = VariableConvention.camelToUser(name);
            return (
              <SettingsOption key={name}>
                <div>{label[0]?.toUpperCase() + label.slice(1)}</div>
                <Input
                  value={value.fancy}
                  onKeyDown={(e) => onChangeVar(name, e)}
                />
              </SettingsOption>
            );
          })
        }
      </SettingsGroup>
    </div>
  );
}
