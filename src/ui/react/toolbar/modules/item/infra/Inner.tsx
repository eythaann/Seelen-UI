import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { computed } from "@preact/signals";
import type { ToolbarItem } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { isEqual } from "lodash";
import { memo } from "preact/compat";
import type { HTMLAttributes } from "preact/compat";
import { useCallback, useMemo } from "preact/compat";

import { EvaluateAction } from "../app/actionEvaluator.ts";
import { $toolbar_state } from "../../shared/state/items.ts";
import { useItemContextMenu } from "./ContextMenu.tsx";
import { ElementsFromEvaluated, StringFromEvaluated, useSandboxedCode } from "./EvaluatedComponents.tsx";
import { useRemoteData } from "../app/hooks/useRemoteData.ts";
import { useFullItemScope } from "../app/hooks/useItemScope.ts";
import { useItemScope } from "../app/hooks/scope.ts";

export interface InnerItemProps {
  module: Omit<ToolbarItem, "type">;
}

const isReorderDisabled = computed(() => $toolbar_state.value.isReorderDisabled);

function InnerItemComponent({ module }: InnerItemProps) {
  const { id, onClick, style, remoteData = {} } = module;

  const { fetching, data: extraVars } = useItemScope(module.scopes);
  const fetchedData = useRemoteData(remoteData);
  const { onContextMenu } = useItemContextMenu(id);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled: isReorderDisabled.value,
    animateLayoutChanges: () => false,
  });

  // Don't render if scope is still loading
  if (fetching) {
    return null;
  }

  const scope = useFullItemScope({
    itemId: id,
    extraVars,
    fetchedData,
  });

  const content = useSandboxedCode({ code: module.template, scope });
  const tooltip = module.tooltip ? useSandboxedCode({ code: module.tooltip, scope }) : null;
  const badge = module.badge ? useSandboxedCode({ code: module.badge, scope }) : null;

  // Memoize callbacks to prevent unnecessary re-renders
  const handleClick = useCallback(() => {
    if (onClick) {
      EvaluateAction(onClick, scope);
    }
  }, [onClick, scope]);

  const handleContextMenu = useCallback((e: MouseEvent) => {
    e.stopPropagation();
    onContextMenu();
  }, [onContextMenu]);

  // Memoize style object
  const itemStyle = useMemo(() => ({
    ...style,
    transform: CSS.Translate.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : 1,
  }), [style, transform, transition, isDragging]);

  // Memoize className
  const itemClassName = useMemo(() =>
    cx("ft-bar-item", {
      "ft-bar-item-clickable": onClick,
    }), [onClick]);

  // Memoize tooltip
  const itemTitle = useMemo(() => tooltip ? StringFromEvaluated({ content: tooltip }) : undefined, [tooltip]);

  if (!content) {
    return null;
  }

  return (
    <div
      id={id}
      ref={setNodeRef}
      {...listeners}
      {...(attributes as HTMLAttributes<HTMLDivElement>)}
      title={itemTitle}
      style={itemStyle}
      className={itemClassName}
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

// Memoize the component to prevent unnecessary re-renders
// Use efficient deep comparison with lodash instead of JSON.stringify
export const InnerItem = memo(InnerItemComponent, (prevProps, nextProps) => {
  return isEqual(prevProps.module, nextProps.module);
});
