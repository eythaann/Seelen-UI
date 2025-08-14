import { Menu } from 'antd';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';
import { PropsWithChildren, useState } from 'react';

import { BackgroundByLayersV2 } from '../../shared/components/BackgroundByLayers/infra';

import { AnimatedDropdown } from '../../shared/components/AnimatedWrappers';
import { useWindowFocusChange } from '../../shared/hooks';

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
    <AnimatedDropdown
      animationDescription={{
        openAnimationName: 'weg-context-menu-container-open',
        closeAnimationName: 'weg-context-menu-container-close',
      }}
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
    </AnimatedDropdown>
  );
}
