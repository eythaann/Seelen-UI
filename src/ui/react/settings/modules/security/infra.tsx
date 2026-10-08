import type { WidgetId, WidgetPerm } from "@seelen-ui/lib/types";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Empty, Switch } from "antd";
import { useTranslation } from "react-i18next";

import { widgets } from "../../state/resources.ts";
import { setWidgetPermission, widgetPermissions } from "../../state/permissions.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";

export function Security() {
  const { t } = useTranslation();

  const entries = Object.entries(widgetPermissions.value).filter(
    ([_, perms]) => perms && Object.keys(perms).length > 0,
  );

  if (!entries.length) {
    return <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={t("security.empty")} />;
  }

  return (
    <>
      {entries.map(([widgetId, perms]) => {
        const widget = widgets.value.find((w) => w.id === widgetId);
        return (
          <SettingsGroup key={widgetId}>
            <SettingsSubGroup
              label={widget ? <ResourceText text={widget.metadata.displayName} /> : widgetId}
            >
              {Object.entries(perms!).map(([perm, state]) => (
                <SettingsOption
                  key={perm}
                  label={t(`security.perms.${perm}`)}
                  action={
                    <Switch
                      value={state === "allowed"}
                      onChange={(allowed) =>
                        setWidgetPermission(
                          widgetId as WidgetId,
                          perm as WidgetPerm,
                          allowed ? "allowed" : "denied",
                        )}
                    />
                  }
                />
              ))}
            </SettingsSubGroup>
          </SettingsGroup>
        );
      })}
    </>
  );
}
