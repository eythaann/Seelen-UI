import {
  Placeholder,
  ToolbarModule,
  ToolbarModuleType,
} from '../../../shared/schemas/Placeholders';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { Reorder } from 'framer-motion';
import { debounce } from 'lodash';
import { JSXElementConstructor, useCallback, useEffect, useRef } from 'react';
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

  const leftRef = useRef<HTMLDivElement>(null);
  const centerRef = useRef<HTMLDivElement>(null);
  const rightRef = useRef<HTMLDivElement>(null);

  const dispatch = useDispatch();

  useEffect(() => {
    if (!leftRef.current || !centerRef.current || !rightRef.current) {
      return;
    }

    leftRef.current.style.width = `calc(50% - ${centerRef.current.offsetWidth / 2}px)`;
    rightRef.current.style.width = `calc(50% - ${centerRef.current.offsetWidth / 2}px)`;
  }, [structure.center]);

  const onReorderPinned = useCallback(
    debounce((apps: (ToolbarModule | string)[]) => {
      let extractedPinned: ToolbarModule[] = [];

      console.log(apps);

      apps.forEach((app) => {
        if (app === DividerStart) {
          dispatch(RootActions.setItemsOnLeft(extractedPinned));
          extractedPinned = [];
          return;
        }

        if (app === DividerEnd) {
          dispatch(RootActions.setItemsOnCenter(extractedPinned));
          extractedPinned = [];
          return;
        }

        if (typeof app !== 'string') {
          extractedPinned.push(app);
        }
      });

      dispatch(RootActions.setItemsOnRight(extractedPinned));
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
      <div className="ft-bar-left" ref={leftRef}>
        {structure.left.map(componentByModule)}
        <Reorder.Item as="div" value={DividerStart} drag={false} style={{ flex: 1 }} />
      </div>
      <div className="ft-bar-center" ref={centerRef}>
        {structure.center.map(componentByModule)}
      </div>
      <div className="ft-bar-right" ref={rightRef}>
        <Reorder.Item as="div" value={DividerEnd} drag={false} style={{ flex: 1 }} />
        {structure.right.map(componentByModule)}
      </div>
    </Reorder.Group>
  );
}
