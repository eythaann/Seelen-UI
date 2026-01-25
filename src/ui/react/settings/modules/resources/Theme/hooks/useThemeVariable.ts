import type { ThemeId } from "@seelen-ui/lib/types";
import { useCallback } from "react";

import { deleteThemeVariable, getThemeVariable, setThemeVariable } from "../application.ts";

export interface UseThemeVariableResult {
  value: string | undefined;
  onChange: (value: string) => void;
  onReset: () => void;
}

export function useThemeVariable(themeId: ThemeId, variableName: string): UseThemeVariableResult {
  const value = getThemeVariable(themeId, variableName);

  const onChange = useCallback(
    (value: string) => {
      setThemeVariable(themeId, variableName, value);
    },
    [themeId, variableName],
  );

  const onReset = useCallback(() => {
    deleteThemeVariable(themeId, variableName);
  }, [themeId, variableName]);

  return { value, onChange, onReset };
}
