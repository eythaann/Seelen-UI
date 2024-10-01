import { Dropdown, Menu } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { GenericToolbarModule, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { SavePlaceholderAsCustom } from '../../main/application';
import { RootActions, Selectors } from '../../shared/store/app';

import { InnerItem, InnerItemProps } from './Inner';

export function Item(props: InnerItemProps) {
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const d = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  return (
    <Dropdown
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      trigger={['contextMenu']}
      dropdownRender={() => (
        <BackgroundByLayersV2 className="ft-bar-item-context-menu-container">
          <Menu
            className="ft-bar-item-context-menu"
            items={[
              {
                key: 'remove',
                label: t('context_menu.remove'),
                className: 'ft-bar-item-context-menu-item',
                onClick() {
                  d(RootActions.removeItem(props.module.id));
                  SavePlaceholderAsCustom()?.catch(console.error);
                },
              },
            ]}
          />
        </BackgroundByLayersV2>
      )}
    >
      <InnerItem {...props} />
    </Dropdown>
  );
}

export function GenericItem({ module }: { module: GenericToolbarModule }) {
  const window = useSelector(Selectors.focused) || {
    name: 'None',
    title: 'No Window Focused',
    exe: null,
  };
  return <Item module={module} extraVars={{ window }} />;
}
