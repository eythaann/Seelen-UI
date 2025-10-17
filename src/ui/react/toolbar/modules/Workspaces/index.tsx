import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useComputed } from "@preact/signals";
import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { type WorkspaceToolbarItem, WorkspaceToolbarItemMode } from "@seelen-ui/lib/types";
import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { useThrottle, useWindowFocusChange } from "@shared/hooks";
import { cx } from "@shared/styles";
import { Menu, Tooltip } from "antd";
import { type HTMLAttributes, useState } from "react";
import { useTranslation } from "react-i18next";

import { CommonItemContextMenu } from "../item/infra/ContextMenu.tsx";
import { BackgroundByLayersV2 } from "@shared/components/BackgroundByLayers/infra";

import { $toolbar_state } from "../shared/state/items.ts";
import { $virtual_desktop } from "../shared/state/system.ts";

interface Props {
  module: WorkspaceToolbarItem;
  onContextMenu?: (e: MouseEvent) => void;
}

let monitorId = Widget.getCurrent().decoded.monitorId!;

function InnerWorkspacesModule({ module, ...rest }: Props) {
  const isReorderDisabled = useComputed(() => $toolbar_state.value.isReorderDisabled);
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: module.id,
    disabled: isReorderDisabled.value,
    animateLayoutChanges: () => false,
  });

  const workspaces = $virtual_desktop.value?.workspaces || [];
  const activeWorkspace = $virtual_desktop.value?.current_workspace;

  const { mode } = module;
  const commonProps = {
    ref: setNodeRef,
    id: module.id,
    ...listeners,
    ...(attributes as HTMLAttributes<HTMLDivElement>),
    style: {
      ...module.style,
      transform: CSS.Translate.toString(transform),
      transition,
      opacity: isDragging ? 0.3 : 1,
    },
  };

  function onContextMenu(e: MouseEvent) {
    rest.onContextMenu?.(e);
    e.stopPropagation();
  }

  const onWheel = useThrottle(
    (isUp: boolean) => {
      const index = workspaces.findIndex((w) => w.id === activeWorkspace);
      const newIndex = isUp ? index - 1 : index + 1;
      if (newIndex >= 0 && newIndex < workspaces.length) {
        invoke(SeelenCommand.SwitchWorkspace, { monitorId, idx: newIndex });
      }
    },
    500,
    { trailing: false },
  );

  if (mode === WorkspaceToolbarItemMode.dotted) {
    return (
      <div
        {...commonProps}
        className="ft-bar-item"
        onContextMenu={onContextMenu}
        onWheel={(e: WheelEvent) => {
          e.stopPropagation();
          onWheel(e.deltaY < 0);
        }}
      >
        <ul className="ft-bar-item-content workspaces">
          {workspaces.map((w, idx) => (
            <li
              key={w.id}
              onClick={() => invoke(SeelenCommand.SwitchWorkspace, { monitorId, idx })}
              className={cx("workspace-dot", {
                "workspace-dot-active": w.id === activeWorkspace,
              })}
            />
          ))}
        </ul>
      </div>
    );
  }

  return (
    <div
      {...commonProps}
      className="ft-bar-group"
      onContextMenu={onContextMenu}
      onWheel={(e: WheelEvent) => {
        e.stopPropagation();
        onWheel(e.deltaY < 0);
      }}
    >
      {workspaces.map((w, idx) => {
        return (
          <Tooltip
            arrow={false}
            mouseLeaveDelay={0}
            classNames={{ root: "ft-bar-item-tooltip" }}
            title={w.name || `Workspace ${idx + 1}`}
            key={w.id}
          >
            <div
              style={module.style}
              className={cx("ft-bar-item", {
                "ft-bar-item-clickable": true,
                "ft-bar-item-active": w.id === activeWorkspace,
              })}
              onClick={() => invoke(SeelenCommand.SwitchWorkspace, { monitorId, idx })}
            >
              <div className="ft-bar-item-content">
                <span>
                  {mode === WorkspaceToolbarItemMode.named ? `${w.name || `Workspace ${idx + 1}`}` : `${idx + 1}`}
                </span>
              </div>
            </div>
          </Tooltip>
        );
      })}
    </div>
  );
}

export function WorkspacesModule({ module }: Props) {
  const [openContextMenu, setOpenContextMenu] = useState(false);

  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  if (!$virtual_desktop.value) {
    return null;
  }

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
            items={CommonItemContextMenu(t, module)}
          />
        </BackgroundByLayersV2>
      )}
    >
      <InnerWorkspacesModule module={module} />
    </AnimatedDropdown>
  );
}
