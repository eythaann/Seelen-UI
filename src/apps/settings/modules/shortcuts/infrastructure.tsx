import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { Switch, Tooltip } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../shared/utils/infra';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';

export function Shortcuts() {
  const ahkEnable = useAppSelector(RootSelectors.ahkEnabled);

  const dispatch = useDispatch();

  const onChange = (value: boolean) => {
    dispatch(RootActions.setAhkEnabled(value));
    dispatch(RootActions.setToBeSaved(true));
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
          <Switch value={ahkEnable} onChange={onChange} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Open this settings window</span>
          <span>Win + K</span>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Focus workspace 1-9</span>
          <span>Alt + 1-9</span>
        </SettingsOption>
        <SettingsOption>
          <span>Move to workspace 1-9</span>
          <span>Alt + Shift + 1-9</span>
        </SettingsOption>
        <SettingsOption>
          <span>Send to workspace 1-9</span>
          <span>Win + Shift + 1-9</span>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Focus">
          <SettingsOption>
            <span>Focus window on left</span>
            <span>Alt + A</span>
          </SettingsOption>
          <SettingsOption>
            <span>Focus window on right</span>
            <span>Alt + D</span>
          </SettingsOption>
          <SettingsOption>
            <span>Focus window on top</span>
            <span>Alt + W</span>
          </SettingsOption>
          <SettingsOption>
            <span>Focus window on bottom</span>
            <span>Alt + S</span>
          </SettingsOption>

          <SettingsOption>
            <span>Cycle next</span>
            <span>Alt + Q</span>
          </SettingsOption>
          <SettingsOption>
            <span>Cycle previous</span>
            <span>Alt + Shif + Q</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Move">
          <SettingsOption>
            <span>window to the left</span>
            <span>Win + Shif + A</span>
          </SettingsOption>
          <SettingsOption>
            <span>window to the right</span>
            <span>Win + Shif + D</span>
          </SettingsOption>
          <SettingsOption>
            <span>window to the top</span>
            <span>Win + Shif + W</span>
          </SettingsOption>
          <SettingsOption>
            <span>window to the bottom</span>
            <span>Win + Shif + S</span>
          </SettingsOption>

          <SettingsOption>
            <span>Flip layout horizontal</span>
            <span>Win + Shif + x</span>
          </SettingsOption>
          <SettingsOption>
            <span>Flip layout vertical</span>
            <span>Win + Shif + z</span>
          </SettingsOption>

          <SettingsOption>
            <span>Promote</span>
            <span>Win + Shif + Enter</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Stack">
          <SettingsOption>
            <span>Stack window on left</span>
            <span>Win + A</span>
          </SettingsOption>
          <SettingsOption>
            <span>Stack window on right</span>
            <span>Win + D</span>
          </SettingsOption>
          <SettingsOption>
            <span>Stack window on top</span>
            <span>Win + W</span>
          </SettingsOption>
          <SettingsOption>
            <span>Stack window on bottom</span>
            <span>Win + S</span>
          </SettingsOption>

          <SettingsOption>
            <span>Unstack</span>
            <span>Win + ;</span>
          </SettingsOption>

          <SettingsOption>
            <span>Cycle stack next</span>
            <span>Win + Q</span>
          </SettingsOption>
          <SettingsOption>
            <span>Cycle stack previous</span>
            <span>Win + Shift + Q</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Resize">
          <SettingsOption>
            <span>Resize horizontal increase</span>
            <span>Alt + =</span>
          </SettingsOption>
          <SettingsOption>
            <span>Resize horizontal decrease</span>
            <span>Alt + -</span>
          </SettingsOption>
          <SettingsOption>
            <span>Resize vertical increase</span>
            <span>Alt + Shift + =</span>
          </SettingsOption>
          <SettingsOption>
            <span>Resize vertical decrease</span>
            <span>Alt + Shift + -</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Toggle float</span>
          <span>Win + F</span>
        </SettingsOption>
        <SettingsOption>
          <span>Toggle monocle</span>
          <span>Win + M</span>
        </SettingsOption>
        <SettingsOption>
          <span>Toggle pause</span>
          <span>Win + Shift + P</span>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
