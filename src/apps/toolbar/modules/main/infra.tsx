import {
  Placeholder,
  ToolbarModule,
  ToolbarModuleType,
} from '../../../shared/schemas/Placeholders';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { Reorder, useForceUpdate } from 'framer-motion';
import { debounce } from 'lodash';
import { JSXElementConstructor, useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../seelenweg/components/BackgrounByLayers/infra';
import { DateModule } from '../Date/infra';
import { DeviceModule } from '../Device/infra';
import { Item } from '../item/infra';
import { MediaModule } from '../media/infra/Module';
import { NetworkModule } from '../network/infra/Module';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SavePlaceholderAsCustom } from './application';

const modulesByType: Record<ToolbarModuleType, JSXElementConstructor<{ module: any }>> = {
  [ToolbarModuleType.Generic]: Item,
  [ToolbarModuleType.Text]: Item,
  [ToolbarModuleType.Date]: DateModule,
  [ToolbarModuleType.Power]: PowerModule,
  [ToolbarModuleType.Settings]: SettingsModule,
  [ToolbarModuleType.Workspaces]: WorkspacesModule,
  [ToolbarModuleType.Tray]: TrayModule,
  [ToolbarModuleType.Network]: NetworkModule,
  [ToolbarModuleType.Media]: MediaModule,
  [ToolbarModuleType.Device]: DeviceModule,
};

interface Props {
  structure: Placeholder;
}

const DividerStart = 'CenterStart';
const DividerEnd = 'CenterEnd';

function componentByModule(module: ToolbarModule) {
  let Component = modulesByType[module.type];
  return <Component key={module.id} module={module} />;
}

export function ToolBar({ structure }: Props) {
  const layers = useSelector(Selectors.themeLayers);

  const dispatch = useDispatch();
  const [forceUpdate] = useForceUpdate();

  const onReorderPinned = useCallback(
    debounce((apps: (ToolbarModule | string)[]) => {
      let dividerStart = apps.indexOf(DividerStart);
      let dividerEnd = apps.indexOf(DividerEnd);

      if (dividerStart === -1 || dividerEnd === -1) {
        forceUpdate();
        return;
      }

      let payload = apps.slice(0, dividerStart) as ToolbarModule[];
      dispatch(RootActions.setItemsOnLeft(payload));

      payload = apps.slice(dividerStart + 1, dividerEnd) as ToolbarModule[];
      dispatch(RootActions.setItemsOnCenter(payload));

      payload = apps.slice(dividerEnd + 1) as ToolbarModule[];
      dispatch(RootActions.setItemsOnRight(payload));

      SavePlaceholderAsCustom();
    }, 10),
    [],
  );

  return (
    <Reorder.Group
      values={[
        ...structure.left,
        DividerStart,
        ...structure.center,
        DividerEnd,
        ...structure.right,
      ]}
      onReorder={onReorderPinned}
      className="ft-bar"
      axis="x"
      as="div"
    >
      <BackgroundByLayers prefix="ft-bar" layers={layers.toolbar.bg} />
      <div className="ft-bar-left">
        {structure.left.map(componentByModule)}
        <Reorder.Item as="div" value={DividerStart} drag={false} style={{ flex: 1 }} />
      </div>
      <div className="ft-bar-center">{structure.center.map(componentByModule)}</div>
      <div className="ft-bar-right">
        <Reorder.Item as="div" value={DividerEnd} drag={false} style={{ flex: 1 }} />
        {structure.right.map(componentByModule)}
      </div>
    </Reorder.Group>
  );
}
