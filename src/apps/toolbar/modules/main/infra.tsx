import { Dropdown } from 'antd';
import { Reorder, useForceUpdate } from 'framer-motion';
import { debounce } from 'lodash';
import { JSXElementConstructor, useCallback, useLayoutEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { HideMode, Plugin } from 'seelen-core';
import { Placeholder, ToolbarModule, ToolbarModuleType } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { DateModule } from '../Date/infra';
import { DeviceModule } from '../Device/infra';
import { GenericItem, Item } from '../item/infra/infra';
import { MediaModule } from '../media/infra/Module';
import { NetworkModule } from '../network/infra/Module';
import { NotificationsModule } from '../Notifications/infra/Module';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SavePlaceholderAsCustom } from './application';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { cx } from '../../../shared/styles';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { MainContextMenu } from './ContextMenu';

const modulesByType: Record<
  ToolbarModuleType,
  JSXElementConstructor<{ module: any; value: any }>
> = {
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

// item can be a toolbar plugin id or a toolbar module
function componentByModule(plugins: Plugin[], item: string | ToolbarModule) {
  let module: ToolbarModule | undefined;

  if (typeof item === 'string') {
    module = plugins.find((p) => p.id === item)?.plugin as ToolbarModule | undefined;
    if (!module) {
      return null;
    }
    module = { ...module };
    module.id = item;
    (module as any).__value__ = item;
  } else {
    module = item;
  }

  let Component = modulesByType[module.type];
  if (!Component) {
    return null;
  }
  return <Component key={module.id} module={module} value={item} />;
}

export function ToolBar({ structure }: Props) {
  const [isAppFocused, setAppFocus] = useState(false);
  const [delayed, setDelayed] = useState(false);
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const plugins = useSelector(Selectors.plugins);
  const isOverlaped = useSelector(Selectors.isOverlaped);
  const hideMode = useSelector(Selectors.settings.hideMode);

  const dispatch = useDispatch();
  const [forceUpdate] = useForceUpdate();

  useWindowFocusChange((focused) => {
    setAppFocus(focused);
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  useLayoutEffect(() => {
    switch (hideMode) {
      case HideMode.Always:
        setDelayed(true);
        break;
      case HideMode.Never:
        setDelayed(false);
        break;
      case HideMode.OnOverlap:
        if (!isOverlaped) {
          setDelayed(false);
          break;
        }
        setTimeout(() => {
          setDelayed(true);
        }, 300);
        break;
    }
  }, [isOverlaped, hideMode]);

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
    !isAppFocused && hideMode !== HideMode.Never && (isOverlaped || hideMode === HideMode.Always);

  return (
    <Dropdown
      trigger={['contextMenu']}
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      dropdownRender={() => <MainContextMenu />}
    >
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
          'ft-bar-delayed': delayed,
        })}
        axis="x"
        as="div"
      >
        <BackgroundByLayersV2 prefix="ft-bar" />
        <div className="ft-bar-left">
          {structure.left.map(componentByModule.bind(null, plugins))}
          <Reorder.Item as="div" value={DividerStart} drag={false} style={{ flex: 1 }} />
        </div>
        <div className="ft-bar-center">
          {structure.center.map(componentByModule.bind(null, plugins))}
        </div>
        <div className="ft-bar-right">
          <Reorder.Item as="div" value={DividerEnd} drag={false} style={{ flex: 1 }} />
          {structure.right.map(componentByModule.bind(null, plugins))}
        </div>
      </Reorder.Group>
    </Dropdown>
  );
}
