import { GenericToolbarItem, TextToolbarItem } from "@seelen-ui/lib/types";
import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { useWindowFocusChange } from "@shared/hooks";
import { Menu } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { Selectors } from "../../shared/store/app";

import { CommonItemContextMenu } from "./ContextMenu";
import { InnerItem, InnerItemProps } from "./Inner";

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
      dropdownRender={() => (
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

export function GenericItem({ module }: { module: GenericToolbarItem }) {
  const window = useSelector(Selectors.focused) || {
    name: "None",
    title: "No Window Focused",
    exe: null,
  };
  return <Item module={module} extraVars={{ window }} />;
}

export function TextItem({ module }: { module: TextToolbarItem }) {
  return <Item module={module} extraVars={{ x: 0 }} />;
}
