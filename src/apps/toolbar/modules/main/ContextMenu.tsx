import { Flex, Menu, Popover } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { invoke, SeelenCommand } from 'seelen-core';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SavePlaceholderAsCustom } from './application';
import { Icon } from 'src/apps/shared/components/Icon';

export function MainContextMenu() {
  const plugins = useSelector(Selectors.plugins);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  return (
    <BackgroundByLayersV2 className="tb-context-menu-container">
      <Menu
        className="tb-context-menu"
        items={[
          {
            key: 'add_module',
            label: (
              <Popover
                trigger={['hover']}
                placement="rightBottom"
                arrow={false}
                content={
                  <BackgroundByLayersV2 className="tb-context-menu-container">
                    <Menu
                      className="tb-context-menu"
                      items={plugins.map((p) => ({
                        key: p.id,
                        label: p.id,
                        onClick: () => {
                          dispatch(RootActions.addItem(p.id));
                          SavePlaceholderAsCustom();
                        },
                      }))}
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
            key: 'task_manager',
            label: t('context_menu.task_manager'),
            onClick() {
              invoke(SeelenCommand.OpenFile, { path: 'C:\\Windows\\System32\\Taskmgr.exe' });
            },
          },
          {
            key: 'settings',
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
