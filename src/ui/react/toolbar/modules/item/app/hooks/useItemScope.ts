import type { WidgetId } from "@seelen-ui/lib/types";
import { useDeepCompareEffect } from "libs/ui/react/utils/hooks.ts";
import { useEffect, useState } from "preact/compat";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";
import { Selectors } from "../../../shared/store/app.ts";
import { triggerWidget } from "../services/widgetTrigger.ts";
import type { ItemScopeOptions } from "../../domain/types.ts";

/**
 * Hook to manage the scope object for toolbar item evaluation.
 * Combines environment variables, translations, extra vars, and fetched data.
 * @param options - Configuration options for the scope
 * @returns The scope object with all combined data
 */
export function useItemScope(options: ItemScopeOptions) {
  const { itemId, extraVars = {}, fetchedData = {} } = options;

  const env = useSelector(Selectors.env);
  const { t } = useTranslation();

  const [scope, setScope] = useState<Record<string, any>>({
    ...extraVars,
    ...fetchedData,
    env,
    t,
    trigger: (widgetId: WidgetId) => triggerWidget(widgetId, itemId),
  });

  // Update env and t when they change
  useEffect(() => {
    setScope((s) => ({ ...s, env, t }));
  }, [env, t]);

  // Update extraVars and fetchedData when they change
  useDeepCompareEffect(() => {
    setScope((s) => ({ ...s, ...extraVars, ...fetchedData }));
  }, [extraVars, fetchedData]);

  // Ensure trigger function always has the latest itemId
  useEffect(() => {
    setScope((s) => ({
      ...s,
      trigger: (widgetId: WidgetId) => triggerWidget(widgetId, itemId),
    }));
  }, [itemId]);

  return scope;
}
