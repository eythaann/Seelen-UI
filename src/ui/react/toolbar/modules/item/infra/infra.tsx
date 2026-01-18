import type { GenericToolbarItem, TextToolbarItem } from "@seelen-ui/lib/types";
import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { useWindowFocusChange } from "libs/ui/react/utils/hooks.ts";
import { Menu } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import { CommonItemContextMenu } from "./ContextMenu.tsx";
import { InnerItem, type InnerItemProps } from "./Inner.tsx";
import { $focused } from "../../shared/state/windows.ts";

export function Item(props: InnerItemProps) {
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  return (
    <AnimatedDropdown
      animationDescription={{
        openAnimationName: "ft-bar-item-context-menu-open",
        closeAnimationName: "ft-bar-item-context-menu-close",
      }}
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      trigger={["contextMenu"]}
      popupRender={() => (
        <BackgroundByLayersV2 className="ft-bar-item-context-menu-container">
          <Menu
            className="ft-bar-item-context-menu"
            items={CommonItemContextMenu(t, props.module)}
          />
        </BackgroundByLayersV2>
      )}
    >
      <InnerItem {...props} clickable={!!props.onClick} />
    </AnimatedDropdown>
  );
}

export function AppsItem({ module, ...rest }: { module: GenericToolbarItem }) {
  const window = $focused.value;
  return <Item {...rest} module={module} extraVars={{ window }} />;
}

export function TextItem({ module, ...rest }: { module: TextToolbarItem }) {
  return <Item {...rest} module={module} />;
}
