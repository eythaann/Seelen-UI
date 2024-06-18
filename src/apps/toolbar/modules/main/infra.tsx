import { Placeholder, ToolbarModule, ToolbarModuleType } from '../../../utils/schemas/Placeholders';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../seelenweg/components/BackgrounByLayers/infra';
import { DateModule } from '../Date/infra';
import { Item } from '../item/infra';
import { NetworkModule } from '../network/infra';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

import { Selectors } from '../shared/store/app';

function componentByModule(module: ToolbarModule, idx: number) {
  switch (module.type) {
    case ToolbarModuleType.Text:
    case ToolbarModuleType.Generic:
      return <Item key={idx} module={module} />;

    case ToolbarModuleType.Date:
      return <DateModule key={idx} module={module} />;

    case ToolbarModuleType.Power:
      return <PowerModule key={idx} module={module} />;

    case ToolbarModuleType.Settings:
      return <SettingsModule key={idx} module={module} />;

    case ToolbarModuleType.Workspaces:
      return <WorkspacesModule key={idx} module={module} />;

    case ToolbarModuleType.Tray:
      return <TrayModule key={idx} module={module} />;

    case ToolbarModuleType.Network:
      return <NetworkModule key={idx} module={module} />;

    default:
      return null;
  }
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
