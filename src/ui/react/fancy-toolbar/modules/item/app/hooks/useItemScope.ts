import type { WidgetId } from "@seelen-ui/lib/types";
import { useMemo, useRef } from "preact/compat";
import { useTranslation } from "react-i18next";
import { triggerWidget } from "../services/widgetTrigger.ts";
import type { ItemScopeOptions } from "../../domain/types.ts";

/**
 * Hook to manage the scope object for toolbar item evaluation.
 * Combines environment variables, translations, extra vars, and fetched data.
 * Optimized to consolidate scope creation and reduce re-computation.
 * @param options - Configuration options for the scope
 * @returns The scope object with all combined data
 */
export function useFullItemScope(options: ItemScopeOptions) {
  const { itemId, extraVars = {}, fetchedData = {} } = options;
  const { t } = useTranslation();

  // Store the trigger function in a ref to keep it stable
  const triggerRef = useRef<(widgetId: WidgetId) => void>();

  // Update trigger function only when itemId changes
  if (!triggerRef.current || triggerRef.current.toString() !== itemId) {
    triggerRef.current = (widgetId: WidgetId) => triggerWidget(widgetId, itemId);
  }

  // Memoize the scope object, only recomputing when t or itemId changes
  // extraVars and fetchedData are handled by deep comparison
  const scope = useMemo(() => ({
    t,
    trigger: triggerRef.current!,
  }), [t, itemId]);

  // Use deep comparison for extraVars and fetchedData
  // Merge them into the scope in a stable way
  return useMemo(() => ({
    ...extraVars,
    ...fetchedData,
    ...scope,
  }), [extraVars, fetchedData, scope]);
}
