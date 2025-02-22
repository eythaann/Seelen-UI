import { HideMode } from '@seelen-ui/lib';
import { ToolbarModuleType as ToolbarItemType } from '@seelen-ui/lib';
import { Plugin, PluginId, ToolbarItem } from '@seelen-ui/lib/types';
import { Reorder, useForceUpdate } from 'framer-motion';
import { JSXElementConstructor, useCallback, useLayoutEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { DateModule } from '../Date/infra';
import { DeviceModule } from '../Device/infra';
import { GenericItem, Item } from '../item/infra/infra';
import { KeyboardModule } from '../Keyboard/infra';
import { MediaModule } from '../media/infra/Module';
import { NetworkModule } from '../network/infra/Module';
import { NotificationsModule } from '../Notifications/infra/Module';
import { PowerModule } from '../Power/infra';
import { SettingsModule } from '../Settings/infra';
import { UserModule } from '../user/infra/Module';

import { RootActions, Selectors } from '../shared/store/app';
import { SaveToolbarItems } from './application';

import { AnimatedDropdown } from '../../../shared/components/AnimatedWrappers';
import { useWindowFocusChange } from '../../../shared/hooks';
import { cx } from '../../../shared/styles';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { MainContextMenu } from './ContextMenu';

const modulesByType: Record<
  ToolbarItem['type'],
  JSXElementConstructor<{ module: any; value: any }>
> = {
  [ToolbarItemType.Text]: Item,
  [ToolbarItemType.Generic]: GenericItem,
  [ToolbarItemType.User]: UserModule,
  [ToolbarItemType.Date]: DateModule,
  [ToolbarItemType.Power]: PowerModule,
  [ToolbarItemType.Keyboard]: KeyboardModule,
  [ToolbarItemType.Settings]: SettingsModule,
  [ToolbarItemType.Workspaces]: WorkspacesModule,
  [ToolbarItemType.Tray]: TrayModule,
  [ToolbarItemType.Network]: NetworkModule,
  [ToolbarItemType.Media]: MediaModule,
  [ToolbarItemType.Device]: DeviceModule,
  [ToolbarItemType.Notifications]: NotificationsModule,
};

const DividerStart = 'CenterStart';
const DividerEnd = 'CenterEnd';

// item can be a toolbar plugin id or a toolbar module
function componentByModule(plugins: Plugin[], item: PluginId | ToolbarItem) {
  let module: ToolbarItem | undefined;

  if (typeof item === 'string') {
    module = plugins.find((p) => p.id === item)?.plugin as ToolbarItem | undefined;
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

export function ToolBar() {
  const structure = useSelector(Selectors.items);

  const [isAppFocused, setAppFocus] = useState(false);
  const [delayed, setDelayed] = useState(false);
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const plugins = useSelector(Selectors.plugins);
  const isOverlaped = useSelector(Selectors.isOverlaped);
  const hideMode = useSelector(Selectors.settings.hideMode);
  const position = useSelector(Selectors.settings.position);

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

  const onReorderPinned = useCallback((apps: (ToolbarItem | string)[]) => {
    let dividerStart = apps.indexOf(DividerStart);
    let dividerEnd = apps.indexOf(DividerEnd);

    if (dividerStart === -1 || dividerEnd === -1) {
      forceUpdate();
      return;
    }

    let payload = apps.slice(0, dividerStart) as ToolbarItem[];
    dispatch(RootActions.setItemsOnLeft(payload));

    payload = apps.slice(dividerStart + 1, dividerEnd) as ToolbarItem[];
    dispatch(RootActions.setItemsOnCenter(payload));

    payload = apps.slice(dividerEnd + 1) as ToolbarItem[];
    dispatch(RootActions.setItemsOnRight(payload));

    SaveToolbarItems()?.catch(console.error);
  }, []);

  const shouldBeHidden =
    !isAppFocused && hideMode !== HideMode.Never && (isOverlaped || hideMode === HideMode.Always);

  return (
    <AnimatedDropdown
      animationDescription={{
        openAnimationName: 'ft-bar-context-menu-open',
        closeAnimationName: 'ft-bar-context-menu-close',
      }}
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
        className={cx('ft-bar', position.toLowerCase(), {
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
    </AnimatedDropdown>
  );
}
