import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useComputed } from "@preact/signals";
import type { ToolbarItem } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import type { HTMLAttributes } from "preact/compat";

import { EvaluateAction } from "../app/actionEvaluator.ts";
import { useItemScope, useRemoteData } from "../app/hooks/index.ts";
import { $toolbar_state } from "../../shared/state/items.ts";
import { useItemContextMenu } from "./ContextMenu.tsx";
import { ElementsFromEvaluated, StringFromEvaluated, useSandboxedCode } from "./EvaluatedComponents.tsx";

export interface InnerItemProps extends HTMLAttributes<HTMLDivElement> {
  module: Omit<ToolbarItem, "type">;
  extraVars?: Record<string, any>;
}

export function InnerItem(props: InnerItemProps) {
  const { extraVars = {}, module, ...rest } = props;

  const { onClick, style, id, remoteData = {} } = module;

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

  const content = useSandboxedCode({ code: module.template, scope });
  const tooltip = module.tooltip ? useSandboxedCode({ code: module.tooltip, scope }) : null;
  const badge = module.badge ? useSandboxedCode({ code: module.badge, scope }) : null;

  if (!content) {
    return null;
  }

  return (
    <div
      {...rest}
      id={id}
      ref={setNodeRef}
      {...listeners}
      {...(attributes as HTMLAttributes<HTMLDivElement>)}
      title={tooltip ? StringFromEvaluated({ content: tooltip }) : undefined}
      style={{
        ...style,
        transform: CSS.Translate.toString(transform),
        transition,
        opacity: isDragging ? 0.3 : 1,
      }}
      className={cx("ft-bar-item", {
        "ft-bar-item-clickable": onClick,
      })}
      onClick={() => {
        if (onClick) {
          EvaluateAction(onClick, scope);
        }
      }}
      onContextMenu={(e: MouseEvent) => {
        e.stopPropagation();
        onContextMenu();
      }}
    >
      <div className="ft-bar-item-content">
        <ElementsFromEvaluated content={content} />
        {!!badge && (
          <div className="ft-bar-item-badge">
            <ElementsFromEvaluated content={badge} />
          </div>
        )}
      </div>
    </div>
  );
}
