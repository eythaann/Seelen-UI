import { Dropdown, Menu } from 'antd';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';
import { PropsWithChildren, useState } from 'react';
import { useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from './BackgroundByLayers/infra';

interface Props extends PropsWithChildren {
  items: ItemType<MenuItemType>[];
}

export function WithContextMenu({ children, items }: Props) {
  const [openContextMenu, setOpenContextMenu] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  return (
    <Dropdown
      placement="topLeft"
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      trigger={['contextMenu']}
      dropdownRender={() => (
        <BackgroundByLayersV2
          className="weg-context-menu-container"
          prefix="menu"
        >
          <Menu
            className="weg-context-menu"
            onMouseMoveCapture={(e) => e.stopPropagation()}
            items={items}
          />
        </BackgroundByLayersV2>
      )}
    >
      {children}
    </Dropdown>
  );
}
