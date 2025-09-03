import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { PluginId } from '@seelen-ui/lib/types';
import { AnimatedPopover } from '@shared/components/AnimatedWrappers';
import { Icon } from '@shared/components/Icon';
import { IconName } from '@shared/components/Icon/icons';
import { Button, Checkbox, Flex, Input, Menu, Space } from 'antd';
import { MenuItemType } from 'antd/es/menu/interface';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { BackgroundByLayersV2 } from '@shared/components/BackgroundByLayers/infra';

import { RestoreToDefault } from './application';

import { $actions, $plugins, $toolbar_state } from '../shared/state/items';

export function MainContextMenu() {
  const [customText, setCustomText] = useState('');

  const { t } = useTranslation();

  const allItems = [...$toolbar_state.value.left, ...$toolbar_state.value.center, ...$toolbar_state.value.right];

  const isAlreadyAdded = (id: PluginId) => {
    return allItems.some((item) => item === id);
  };

  function addCustomTextToToolbar() {
    $actions.addTextItem(customText);
    setCustomText('');
  }

  return (
    <BackgroundByLayersV2 className="tb-context-menu-container">
      <Menu
        className="tb-context-menu"
        items={[
          {
            key: 'add_module',
            icon: <Icon iconName="CgExtensionAdd" />,
            label: (
              <AnimatedPopover
                trigger="hover"
                placement="right"
                content={
                  <BackgroundByLayersV2 className="tb-context-menu-container">
                    <Menu
                      className="tb-context-menu"
                      items={[
                        {
                          key: 'restore',
                          icon: <Icon iconName="TbRestore" />,
                          label: t('context_menu.restore'),
                          onClick() {
                            RestoreToDefault();
                          },
                        },
                        {
                          type: 'divider',
                        },
                        {
                          key: 'custom-text',
                          label: (
                            <Space.Compact block>
                              <Input
                                placeholder={t('context_menu.add_custom_text')}
                                value={customText}
                                onChange={(e) => setCustomText(e.currentTarget.value)}
                                onKeyDown={(e) => {
                                  if (e.key === 'Enter') {
                                    addCustomTextToToolbar();
                                  }
                                }}
                              />
                              <Button type="primary" onClick={addCustomTextToToolbar}>
                                <Icon iconName="MdOutlineTextFields" />
                              </Button>
                            </Space.Compact>
                          ),
                        },
                        {
                          type: 'divider',
                        },
                        ...$plugins.value
                          .toSorted((p1, p2) => p1.id.localeCompare(p2.id))
                          .map<MenuItemType>((plugin) => {
                          const added = isAlreadyAdded(plugin.id);
                          return {
                            key: plugin.id,
                            label: (
                              <div className="tb-context-menu-module-item">
                                <Icon iconName={plugin.icon as IconName} />
                                <Checkbox checked={added} />
                                <span className="tb-context-menu-module-item-text">
                                  {plugin.id}
                                </span>
                              </div>
                            ),
                            onClick: () => {
                              if (added) {
                                $actions.removeItem(plugin.id);
                              } else {
                                $actions.addItem(plugin.id);
                              }
                            },
                          };
                        }),
                      ]}
                    />
                  </BackgroundByLayersV2>
                }
              >
                <Flex justify="space-between" align="center">
                  {t('context_menu.modules')}
                  <Icon iconName="FaChevronRight" size={12} />
                </Flex>
              </AnimatedPopover>
            ),
          },
          {
            type: 'divider',
          },
          {
            key: 'reoder',
            icon: <Icon iconName={$toolbar_state.value.isReorderDisabled ? 'VscUnlock' : 'VscLock'} />,
            label: t(
              $toolbar_state.value.isReorderDisabled
                ? 'context_menu.reorder_enable'
                : 'context_menu.reorder_disable',
            ),
            onClick() {
              $toolbar_state.value = {
                ...$toolbar_state.value,
                isReorderDisabled: !$toolbar_state.value.isReorderDisabled,
              };
            },
          },
          {
            key: 'task_manager',
            icon: <Icon iconName="PiChartLineFill" />,
            label: t('context_menu.task_manager'),
            onClick() {
              invoke(SeelenCommand.OpenFile, { path: 'Taskmgr.exe' });
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
