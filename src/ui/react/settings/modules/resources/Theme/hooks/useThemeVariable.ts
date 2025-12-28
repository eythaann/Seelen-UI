import type { ThemeId } from "@seelen-ui/lib/types";
import { useCallback } from "react";
import { useDispatch, useSelector } from "react-redux";

import { RootActions } from "../../../shared/store/app/reducer.ts";
import type { RootState } from "../../../shared/store/domain.ts";

export interface UseThemeVariableResult {
  value: string | undefined;
  onChange: (value: string) => void;
  onReset: () => void;
}

export function useThemeVariable(themeId: ThemeId, variableName: string): UseThemeVariableResult {
  const dispatch = useDispatch();

  const value = useSelector((state: RootState) => state.byTheme[themeId]?.[variableName]);

  const onChange = useCallback(
    (value: string) => {
      dispatch(RootActions.setThemeVariable({ themeId, name: variableName, value }));
    },
    [dispatch, themeId, variableName],
  );

  const onReset = useCallback(() => {
    dispatch(RootActions.deleteThemeVariable({ themeId, name: variableName }));
  }, [dispatch, themeId, variableName]);

  return { value, onChange, onReset };
}
