import { Placeholder, ToolbarModule, ToolbarModuleType } from '../../../shared/schemas/Placeholders';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { JSXElementConstructor } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../seelenweg/components/BackgrounByLayers/infra';
import { DateModule } from '../Date/infra';
import { DeviceModule } from '../Device/infra';
import { Item } from '../item/infra';
import { MediaModule } from '../media/infra/Module';
import { NetworkModule } from '../network/infra/Module';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

import { Selectors } from '../shared/store/app';

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

function componentByModule(module: ToolbarModule, idx: number) {
  let Component = modulesByType[module.type];
  return <Component key={idx} module={module} />;
}

interface Props {
  structure: Placeholder;
}

export function ToolBar({ structure }: Props) {
  const layers = useSelector(Selectors.themeLayers);

  return (
    <div className="ft-bar">
      <BackgroundByLayers prefix="ft-bar" layers={layers.toolbar.bg} />
      <div className="ft-bar-left">{structure.left.map(componentByModule)}</div>
      <div className="ft-bar-center">{structure.center.map(componentByModule)}</div>
      <div className="ft-bar-right">{structure.right.map(componentByModule)}</div>
    </div>
  );
}
