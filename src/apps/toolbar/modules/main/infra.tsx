import { ToolbarModuleType as ToolbarItemType } from '@seelen-ui/lib';
import { Plugin, PluginId, ToolbarItem } from '@seelen-ui/lib/types';
import { Reorder, useForceUpdate } from 'framer-motion';
import { isEqual } from 'lodash';
import { AnyComponent } from 'preact';
import { memo, useCallback, useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';
import { BluetoothModule } from '../bluetooth/infra/Module';
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

import { RootState } from '../shared/store/domain';

import { AnimatedDropdown } from '../../../shared/components/AnimatedWrappers';
import { useWindowFocusChange } from '../../../shared/hooks';
import { cx } from '../../../shared/styles';
import { $bar_should_be_hidden, $settings } from '../shared/state/mod';
import { TrayModule } from '../Tray';
import { WorkspacesModule } from '../Workspaces';
import { MainContextMenu } from './ContextMenu';

const modulesByType: Record<ToolbarItem['type'], AnyComponent<{ module: any; value: any }>> = {
  [ToolbarItemType.Text]: memo(Item),
  [ToolbarItemType.Generic]: memo(GenericItem),
  [ToolbarItemType.User]: memo(UserModule),
  [ToolbarItemType.Date]: memo(DateModule),
  [ToolbarItemType.Power]: memo(PowerModule),
  [ToolbarItemType.Keyboard]: memo(KeyboardModule),
  [ToolbarItemType.Settings]: memo(SettingsModule),
  [ToolbarItemType.Workspaces]: memo(WorkspacesModule),
  [ToolbarItemType.Tray]: memo(TrayModule),
  [ToolbarItemType.Bluetooth]: memo(BluetoothModule),
  [ToolbarItemType.Network]: memo(NetworkModule),
  [ToolbarItemType.Media]: memo(MediaModule),
  [ToolbarItemType.Device]: memo(DeviceModule),
  [ToolbarItemType.Notifications]: memo(NotificationsModule),
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
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const structure = useSelector(Selectors.items);
  const focusedWindow = useSelector(Selectors.focused);
  const plugins = useSelector(Selectors.plugins);

  const data = useBarData();

  const dispatch = useDispatch();
  const [forceUpdate] = useForceUpdate();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

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
        as="div"
        axis="x"
        values={[
          ...structure.left,
          DividerStart,
          ...structure.center,
          DividerEnd,
          ...structure.right,
        ]}
        onReorder={onReorderPinned}
        className={cx('ft-bar', $settings.value.position.toLowerCase(), {
          'ft-bar-hidden': $bar_should_be_hidden.value,
        })}
        data-there-is-maximized-on-background={data.thereIsMaximizedOnBg}
        data-focused-is-maximized={!!focusedWindow?.isMaximized}
        data-focused-is-overlay={!!focusedWindow?.isSeelenOverlay}
        data-dynamic-color={$settings.value.dynamicColor}
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

function useBarData() {
  const maximizedOnBg = useSelector((state: RootState) => {
    return state.openApps.find((app) => app.isZoomed && !app.isIconic);
  });

  const colors = useSelector(Selectors.windowColorByHandle, isEqual);
  const color = maximizedOnBg ? colors[String(maximizedOnBg.handle)] : undefined;

  useEffect(() => {
    if (color) {
      document.documentElement.style.setProperty(
        '--color-maximized-on-bg-background',
        color.background,
      );
      document.documentElement.style.setProperty(
        '--color-maximized-on-bg-foreground',
        color.foreground,
      );
    } else {
      document.documentElement.style.removeProperty('--color-maximized-on-bg-background');
      document.documentElement.style.removeProperty('--color-maximized-on-bg-foreground');
    }
  }, [color]);

  return {
    thereIsMaximizedOnBg: !!maximizedOnBg,
    dynamicBarColor: color,
  };
}
