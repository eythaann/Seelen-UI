import { Icon } from '../../../utils/components/Icon';
import { useSelector } from 'react-redux';

import { DateModule } from '../Date/infra';

import { Selectors } from '../shared/store/app';

export function ToolBar() {
  const focused = useSelector(Selectors.focused);

  return (
    <div className="ft-bar">
      <div className="ft-bar-left">
        <Icon lib="md" iconName="MdWindow" propsIcon={{ size: 14 }} />
        <span>ßeta</span>
        <span>&nbsp;|&nbsp;</span>
        <span>{focused?.name} - {focused?.title}</span>
      </div>
      <div className="ft-bar-center">
        <DateModule />
      </div>
      <div className="ft-bar-right">
        <span>Incoming:&nbsp;</span>
        <Icon lib="bs" iconName="BsBluetooth" propsIcon={{ size: 14 }} />
        <Icon lib="bs" iconName="BsWifi" />
        <Icon lib="pi" iconName="PiBatteryFullFill" />
        <Icon lib="lu" iconName="LuSettings2" />
      </div>
    </div>
  );
}
