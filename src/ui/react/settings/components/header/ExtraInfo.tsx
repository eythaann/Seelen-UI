import { Tooltip } from "antd";
import type { AnyComponent } from "preact";
import { useTranslation } from "react-i18next";

import { RoutePath } from "../navigation/routes.tsx";

export const RouteExtraInfo: { [key: string]: AnyComponent } = {
  [RoutePath.SettingsByApplication]: () => {
    const { t } = useTranslation();
    return (
      <Tooltip title={t("apps_configurations.extra_info")}>
        <span>🛈</span>
      </Tooltip>
    );
  },
};
