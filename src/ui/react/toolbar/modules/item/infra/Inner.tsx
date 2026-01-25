import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useComputed } from "@preact/signals";
import type { ToolbarItem } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { Tooltip } from "antd";
import type { HTMLAttributes } from "preact/compat";

import { EvaluateAction } from "../app/actionEvaluator.ts";
import { useItemScope, useRemoteData } from "../app/hooks/index.ts";
import { $toolbar_state } from "../../shared/state/items.ts";
import { SanboxedComponent } from "./EvaluatedComponents.tsx";
import { useItemContextMenu } from "./ContextMenu.tsx";

export interface InnerItemProps extends HTMLAttributes<HTMLDivElement> {
  module: Omit<ToolbarItem, "type">;
  extraVars?: Record<string, any>;
  onClick?: (e: MouseEvent) => void;
}

export function InnerItem(props: InnerItemProps) {
  const {
    extraVars = {},
    module,
    onClick: onClickProp,
    children,
    ...rest
  } = props;

  const { template, tooltip, onClickV2, style, id, badge, remoteData = {} } = module;

  const { onContextMenu } = useItemContextMenu(id);

  const fetchedData = useRemoteData(remoteData);
  const isReorderDisabled = useComputed(() => $toolbar_state.value.isReorderDisabled);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled: isReorderDisabled.value,
    animateLayoutChanges: () => false,
  });

  const scope = useItemScope({
    itemId: id,
    extraVars,
    fetchedData,
  });

  return (
    <Tooltip
      arrow={false}
      mouseLeaveDelay={0}
      classNames={{ root: "ft-bar-item-tooltip" }}
      title={tooltip ? <SanboxedComponent code={tooltip} scope={scope} /> : undefined}
    >
      <div
        {...rest}
        id={id}
        ref={setNodeRef}
        {...listeners}
        {...(attributes as HTMLAttributes<HTMLDivElement>)}
        style={{
          ...style,
          transform: CSS.Translate.toString(transform),
          transition,
          opacity: isDragging ? 0.3 : 1,
        }}
        className={cx("ft-bar-item", {
          "ft-bar-item-clickable": onClickProp || onClickV2,
        })}
        onClick={(e: MouseEvent) => {
          onClickProp?.(e);
          if (onClickV2) {
            EvaluateAction(onClickV2, scope);
          }
        }}
        onContextMenu={(e: MouseEvent) => {
          e.stopPropagation();
          onContextMenu();
        }}
      >
        <div className="ft-bar-item-content">
          {children || <SanboxedComponent code={template} scope={scope} />}
          {!!badge && (
            <div className="ft-bar-item-badge">
              <SanboxedComponent code={badge} scope={scope} />
            </div>
          )}
        </div>
      </div>
    </Tooltip>
  );
}
