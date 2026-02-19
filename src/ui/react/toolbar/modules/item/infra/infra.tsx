import { useSortable } from "@dnd-kit/react/sortable";
import { computed } from "@preact/signals";
import type { ToolbarItem } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback, useMemo } from "preact/compat";
import type { Ref } from "preact";

import { EvaluateAction } from "../app/actionEvaluator.ts";
import { $toolbar_state } from "../../shared/state/items.ts";
import { useItemContextMenu } from "./ContextMenu.tsx";
import { ElementsFromEvaluated, StringFromEvaluated, useSandboxedCode } from "./EvaluatedComponents.tsx";
import { useRemoteData } from "../app/hooks/useRemoteData.ts";
import { useFullItemScope } from "../app/hooks/useItemScope.ts";
import { useItemScope } from "../app/hooks/scope.ts";
import { RestrictToHorizontalAxis } from "@dnd-kit/abstract/modifiers";

export interface ItemProps {
  module: ToolbarItem;
  index: number;
  group: string;
}

interface InnerItemProps {
  module: ToolbarItem;
  extraVars: Record<string, unknown>;
  nodeRef?: Ref<HTMLDivElement>;
  isDragging?: boolean;
}

interface SortableInnerItemProps extends ItemProps {
  extraVars: Record<string, unknown>;
}

// Module-level computed signal (read only in the signal-reactive layer below)
const isReorderDisabled = computed(() => $toolbar_state.value.isReorderDisabled);

function InnerItem({ module, extraVars, nodeRef, isDragging = false }: InnerItemProps) {
  const { id, onClick, style, remoteData = {} } = module;

  const fetchedData = useRemoteData(remoteData);
  const { onContextMenu } = useItemContextMenu(id);

  const scope = useFullItemScope({
    itemId: id,
    extraVars,
    fetchedData,
  });

  const content = useSandboxedCode({ code: module.template, scope });
  const tooltip = module.tooltip ? useSandboxedCode({ code: module.tooltip, scope }) : null;
  const badge = module.badge ? useSandboxedCode({ code: module.badge, scope }) : null;

  const handleClick = useCallback(() => {
    if (onClick) {
      EvaluateAction(onClick, scope);
    }
  }, [onClick, scope]);

  const handleContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      onContextMenu();
    },
    [onContextMenu],
  );

  const itemStyle = {
    ...style,
    opacity: isDragging ? 0.3 : 1,
  };

  const itemTitle = useMemo(
    () => (tooltip ? StringFromEvaluated({ content: tooltip }) : undefined),
    [tooltip],
  );

  if (!content) {
    return null;
  }

  return (
    <div
      id={id}
      ref={nodeRef}
      data-dragging={isDragging}
      title={itemTitle}
      style={itemStyle}
      className={cx("ft-bar-item", {
        "ft-bar-item-clickable": onClick,
      })}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
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

function SortableInnerItem({ module, index, group, extraVars }: SortableInnerItemProps) {
  const { ref, isDragging } = useSortable({
    id: module.id,
    index,
    type: "item",
    accept: "item",
    group,
    disabled: isReorderDisabled.value,
    modifiers: [RestrictToHorizontalAxis],
  });

  return (
    <InnerItem
      module={module}
      extraVars={extraVars}
      nodeRef={ref as Ref<HTMLDivElement>}
      isDragging={isDragging}
    />
  );
}

export function SortableItem({ module, index, group }: ItemProps) {
  const { fetching, data: extraVars } = useItemScope(module.scopes);
  if (fetching) {
    return null;
  }
  return <SortableInnerItem module={module} index={index} group={group} extraVars={extraVars} />;
}

export function Item({ module }: ItemProps) {
  const { fetching, data: extraVars } = useItemScope(module.scopes);
  if (fetching) {
    return null;
  }
  return <InnerItem module={module} extraVars={extraVars} />;
}
