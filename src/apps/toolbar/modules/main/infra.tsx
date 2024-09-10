import { Reorder, useForceUpdate } from 'framer-motion';
import { debounce } from 'lodash';
import { JSXElementConstructor, useCallback, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { HideMode, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { DateModule } from '../Date/infra';
import { DeviceModule } from '../Device/infra';
import { GenericItem, Item } from '../item/infra';
import { MediaModule } from '../media/infra/Module';
import { NetworkModule } from '../network/infra/Module';
import { NotificationsModule } from '../Notifications/infra/Module';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SavePlaceholderAsCustom } from './application';

import {
  Placeholder,
  ToolbarModule,
  ToolbarModuleType,
} from '../../../shared/schemas/Placeholders';
import { cx } from '../../../shared/styles';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';

const modulesByType: Record<ToolbarModuleType, JSXElementConstructor<{ module: any }>> = {
  [ToolbarModuleType.Text]: Item,
  [ToolbarModuleType.Generic]: GenericItem,
  [ToolbarModuleType.Date]: DateModule,
  [ToolbarModuleType.Power]: PowerModule,
  [ToolbarModuleType.Settings]: SettingsModule,
  [ToolbarModuleType.Workspaces]: WorkspacesModule,
  [ToolbarModuleType.Tray]: TrayModule,
  [ToolbarModuleType.Network]: NetworkModule,
  [ToolbarModuleType.Media]: MediaModule,
  [ToolbarModuleType.Device]: DeviceModule,
  [ToolbarModuleType.Notifications]: NotificationsModule,
};

interface Props {
  structure: Placeholder;
}

const DividerStart = 'CenterStart';
const DividerEnd = 'CenterEnd';

function componentByModule(module: ToolbarModule, idx: number) {
  let Component = modulesByType[module.type];
  if (!Component) {
    return null;
  }
  return <Component key={module.id || module.template || idx} module={module} />;
}

export function ToolBar({ structure }: Props) {
  const [isAppFocused, setAppFocus] = useState(false);
  const isOverlaped = useSelector(Selectors.isOverlaped);
  const hideMode = useSelector(Selectors.settings.hideMode);

  const dispatch = useDispatch();
  const [forceUpdate] = useForceUpdate();

  useWindowFocusChange((focused) => {
    setAppFocus(focused);
  });

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

      SavePlaceholderAsCustom()?.catch(console.error);
    }, 10),
    [],
  );

  const shouldBeHidden =
    !isAppFocused &&
    hideMode !== HideMode.Never &&
    (isOverlaped || hideMode === HideMode.Always);

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
      className={cx('ft-bar', {
        'ft-bar-hidden': shouldBeHidden,
      })}
      axis="x"
      as="div"
    >
      <BackgroundByLayersV2 prefix="ft-bar" />
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
