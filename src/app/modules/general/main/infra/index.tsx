import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from '../../../../components/SettingsBox';
import { InputNumber, Select, Switch } from 'antd';
import { useSelector } from 'react-redux';

import { AnimationsSettings } from '../../animations/infra';
import { BorderSettings } from '../../border/infra';
import { ContainerTopBarSettings } from '../../containerTopBar/infra';

import { GeneralSettingsSelectors } from '../../../shared/app/selectors';

import {
  CrossMonitorMoveBehaviour,
  FocusFollowsMouse,
  UnmanagedWindowOperationBehaviour,
  WindowContainerBehaviour,
  WindowHidingBehaviour,
} from '../domain';

export function General() {
  const containerPadding = useSelector(GeneralSettingsSelectors.containerPadding);
  const workspacePadding = useSelector(GeneralSettingsSelectors.workspacePadding);
  const workAreaOffset = useSelector(GeneralSettingsSelectors.globalWorkAreaOffset);

  const autoStackinByCategory = useSelector(GeneralSettingsSelectors.autoStackinByCategory);
  const windowContainerBehaviour = useSelector(GeneralSettingsSelectors.windowContainerBehaviour);
  const windowHidingBehaviour = useSelector(GeneralSettingsSelectors.windowHidingBehaviour);

  const invisibleBorders = useSelector(GeneralSettingsSelectors.invisibleBorders);

  const focusFollowsMouse = useSelector(GeneralSettingsSelectors.focusFollowsMouse);
  const mouseFollowFocus = useSelector(GeneralSettingsSelectors.mouseFollowFocus);

  const resizeDelta = useSelector(GeneralSettingsSelectors.resizeDelta);
  const unmanagedWindowOpBehaviour = useSelector(
    GeneralSettingsSelectors.unmanagedWindowOperationBehaviour,
  );
  const crossMonitorMoveBehaviour = useSelector(GeneralSettingsSelectors.crossMonitorMoveBehaviour);

  return (
    <div>
      <SettingsGroup>
        <BorderSettings />
        <div>
          <SettingsOption>
            <span>Mouse follows focus</span>
            <Switch value={mouseFollowFocus} />
          </SettingsOption>
          <SettingsOption>
            <span>Focus follows mouse mode</span>
            <Select
              value={focusFollowsMouse}
              options={Object.values(FocusFollowsMouse).map((op) => ({
                label: op,
              }))}
            />
          </SettingsOption>
        </div>
      </SettingsGroup>

      <ContainerTopBarSettings />

      <SettingsGroup>
        <div>
          <SettingsOption>
            <span>Default container padding</span>
            <InputNumber value={containerPadding} />
          </SettingsOption>
          <SettingsOption>
            <span>Default workspace padding</span>
            <InputNumber value={workspacePadding} />
          </SettingsOption>
        </div>
        <SettingsSubGroup label="Global work area offset">
          <SettingsOption>
            <span>Left</span>
            <InputNumber value={workAreaOffset.left} />
          </SettingsOption>
          <SettingsOption>
            <span>Top</span>
            <InputNumber value={workAreaOffset.top} />
          </SettingsOption>
          <SettingsOption>
            <span>Right</span>
            <InputNumber value={workAreaOffset.right} />
          </SettingsOption>
          <SettingsOption>
            <span>Bottom</span>
            <InputNumber value={workAreaOffset.bottom} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <div>
          <SettingsOption>
            <span>Window container behaviour</span>
            <Select
              value={windowContainerBehaviour}
              options={Object.values(WindowContainerBehaviour).map((op) => ({
                label: op,
              }))}
            />
          </SettingsOption>
          <SettingsOption>
            <span>Auto Stack by category (append if same category)</span>
            <Switch value={autoStackinByCategory} />
          </SettingsOption>
        </div>
        <SettingsOption>
          <span>Window hiding behaviour</span>
          <Select
            value={windowHidingBehaviour}
            options={Object.values(WindowHidingBehaviour).map((op) => ({
              label: op,
            }))}
          />
        </SettingsOption>
      </SettingsGroup>

      <AnimationsSettings />

      <SettingsGroup>
        <SettingsSubGroup label="Invisible borders">
          <SettingsOption>
            <span>Left</span>
            <InputNumber value={invisibleBorders.left} />
          </SettingsOption>
          <SettingsOption>
            <span>Top</span>
            <InputNumber value={invisibleBorders.top} />
          </SettingsOption>
          <SettingsOption>
            <span>Right</span>
            <InputNumber value={invisibleBorders.right} />
          </SettingsOption>
          <SettingsOption>
            <span>Bottom</span>
            <InputNumber value={invisibleBorders.bottom} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Resize delta</span>
          <InputNumber value={resizeDelta} />
        </SettingsOption>
        <SettingsOption>
          <span>Cross monitor move behaviour - Swap Insert</span>
          <Select
            value={crossMonitorMoveBehaviour}
            options={Object.values(CrossMonitorMoveBehaviour).map((op) => ({
              label: op,
            }))}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Unmanaged window operation behaviour</span>
          <Select
            value={unmanagedWindowOpBehaviour}
            options={Object.values(UnmanagedWindowOperationBehaviour).map((op) => ({
              label: op,
            }))}
          />
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
