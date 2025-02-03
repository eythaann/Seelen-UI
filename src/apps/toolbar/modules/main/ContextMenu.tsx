import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { PluginId } from '@seelen-ui/lib/types';
import { Flex, Menu, Popover } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SaveToolbarItems } from './application';
import { Icon } from 'src/apps/shared/components/Icon';

export function MainContextMenu() {
  const items = useSelector(Selectors.items);
  const plugins = useSelector(Selectors.plugins);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const allItems = [...items.left, ...items.center, ...items.right];

  const isAlreadyAdded = (id: PluginId) => {
    return allItems.some((item) => item === id);
  };

  return (
    <BackgroundByLayersV2 className="tb-context-menu-container">
      <Menu
        className="tb-context-menu"
        items={[
          {
            key: 'add_module',
            icon: <Icon iconName="CgExtensionAdd" />,
            label: (
              <Popover
                trigger={['hover']}
                placement="rightBottom"
                arrow={false}
                content={
                  <BackgroundByLayersV2 className="tb-context-menu-container">
                    <Menu
                      className="tb-context-menu"
                      items={plugins.map((plugin) => {
                        const added = isAlreadyAdded(plugin.id);
                        return {
                          key: plugin.id,
                          icon: <Icon iconName={plugin.icon} />,
                          label: added ? `✓ ${plugin.id}` : plugin.id,
                          onClick: () => {
                            if (added) {
                              dispatch(RootActions.removeItem(plugin.id));
                            } else {
                              dispatch(RootActions.addItem(plugin.id));
                            }
                            SaveToolbarItems();
                          },
                        };
                      })}
                    />
                  </BackgroundByLayersV2>
                }
              >
                <Flex justify="space-between" align="center">
                  {t('context_menu.add_module')}
                  <Icon iconName="FaChevronRight" size={12} />
                </Flex>
              </Popover>
            ),
          },
          {
            type: 'divider',
          },
          {
            key: 'reoder',
            icon: <Icon iconName={!items.isReorderDisabled ? 'VscLock' : 'VscUnlock' } />,
            label: t(!items.isReorderDisabled ? 'context_menu.reorder_disable' : 'context_menu.reorder_enable' ),
            onClick() {
              dispatch(RootActions.setToolbarReorderDisabled(!items.isReorderDisabled));
              SaveToolbarItems();
            },
          },
          {
            key: 'task_manager',
            icon: <Icon iconName="PiChartLineFill" />,
            label: t('context_menu.task_manager'),
            onClick() {
              invoke(SeelenCommand.OpenFile, { path: 'C:\\Windows\\System32\\Taskmgr.exe' });
            },
          },
          {
            key: 'settings',
            icon: <Icon iconName="RiSettings4Fill" />,
            label: t('context_menu.settings'),
            onClick() {
              invoke(SeelenCommand.ShowAppSettings);
            },
          },
        ]}
      />
    </BackgroundByLayersV2>
  );
}
