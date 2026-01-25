import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { useWindowFocusChange } from "libs/ui/react/utils/hooks";
import { Menu } from "antd";
import type { ItemType, MenuItemType } from "antd/es/menu/interface";
import { type PropsWithChildren, useState } from "react";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra";

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
        openAnimationName: "weg-context-menu-container-open",
        closeAnimationName: "weg-context-menu-container-close",
      }}
      placement="topLeft"
      open={openContextMenu}
      onOpenChange={(isOpen) => {
        setOpenContextMenu(isOpen);
        if (onOpenChange) {
          onOpenChange(isOpen);
        }
      }}
      trigger={["contextMenu"]}
      popupRender={() => (
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
