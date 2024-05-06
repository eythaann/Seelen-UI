import { Placeholder, ToolbarModule, ToolbarModuleType } from '../../../utils/schemas/Placeholders';

import { DateModule } from '../Date/infra';
import { Item } from '../item/infra';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

function componentByModule(module: ToolbarModule, idx: number) {
  switch (module.type) {
    case ToolbarModuleType.TEXT:
    case ToolbarModuleType.GENERIC:
      return <Item key={idx} module={module} />;
    case ToolbarModuleType.DATE:
      return <DateModule key={idx} module={module} />;
    case ToolbarModuleType.POWER:
      return <PowerModule key={idx} module={module} />;
    case ToolbarModuleType.SETTINGS:
      return <SettingsModule key={idx} module={module} />;
    default:
      return null;
  }
}

interface Props {
  structure: Placeholder;
}

export function ToolBar({ structure }: Props) {
  return (
    <div className="ft-bar">
      <div className="ft-bar-left">{structure.left.map(componentByModule)}</div>
      <div className="ft-bar-center">{structure.center.map(componentByModule)}</div>
      <div className="ft-bar-right">{structure.right.map(componentByModule)}</div>
    </div>
  );
}
