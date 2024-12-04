import { Dropdown, Menu } from 'antd';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';
import { PropsWithChildren, useState } from 'react';

import { BackgroundByLayersV2 } from './BackgroundByLayers/infra';

import { useWindowFocusChange } from 'src/apps/shared/hooks';

interface Props extends PropsWithChildren {
  items: ItemType<MenuItemType>[];
  onOpenChange?: (isOpen: boolean) => void;
}

export function WithContextMenu({ children, items, onOpenChange }: Props) {
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
      onOpenChange={(isOpen) => {
        setOpenContextMenu(isOpen);
        if (onOpenChange) {
          onOpenChange(isOpen);
        }
      }}
      trigger={['contextMenu']}
      dropdownRender={() => (
        <BackgroundByLayersV2
          className="weg-context-menu-container"
          prefix="menu"
          onContextMenu={(e) => {
            e.stopPropagation();
            e.preventDefault();
          }}
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
