import { useComputed } from "@preact/signals";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Modal, Select } from "antd";
import { type ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";

import { $virtual_desktops } from "../../shared/signals";
import { setMonitorWallpaperCollection, setWorkspaceWallpaperCollection } from "../../Wall/application.ts";
import { settings } from "../../../state/mod.ts";
import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";

interface Props {
  monitorId: string;
  title: ReactNode;
}

export function WallpaperSettingsModal({ monitorId, title }: Props) {
  const [open, setOpen] = useState(false);
  const wallpaperCollections = settings.value.wallpaperCollections;
  const monitorsV3 = settings.value.monitorsV3;
  const { t } = useTranslation();

  const monitorConfig = monitorsV3[monitorId];
  const selectedCollection = monitorConfig?.wallpaperCollection ?? null;

  // Get workspaces for this monitor from virtual desktops signal
  const monitorWorkspaces = useComputed(() => {
    return $virtual_desktops.value?.monitors[monitorId]?.workspaces || [];
  });

  return (
    <>
      <Modal
        open={open}
        onCancel={() => setOpen(false)}
        title={title}
        footer={null}
        centered
        width={600}
      >
        <SettingsGroup>
          <SettingsOption>
            <b>{t("wall.monitor_collection")}</b>
            <Select
              style={{ width: 300 }}
              value={selectedCollection ?? undefined}
              onChange={(value) => setMonitorWallpaperCollection(monitorId, value || null)}
              placeholder={t("inherit")}
              allowClear
            >
              {wallpaperCollections.map((collection) => (
                <Select.Option key={collection.id} value={collection.id}>
                  {collection.name}
                </Select.Option>
              ))}
            </Select>
          </SettingsOption>
        </SettingsGroup>

        {monitorWorkspaces.value.length > 0 && (
          <SettingsGroup>
            <div style={{ marginBottom: 12 }}>
              <b>{t("wall.workspace_collections")}</b>
            </div>
            {monitorWorkspaces.value.map((workspace, idx) => {
              const workspaceConfig = monitorConfig?.byWorkspace?.[workspace.id];
              const workspaceCollection = workspaceConfig?.wallpaperCollection ?? null;

              return (
                <SettingsOption key={workspace.id}>
                  <span>{workspace.name || `Workspace ${idx + 1}`}</span>
                  <Select
                    style={{ width: 300 }}
                    value={workspaceCollection ?? undefined}
                    onChange={(value) => setWorkspaceWallpaperCollection(monitorId, workspace.id, value || null)}
                    placeholder={t("inherit")}
                    allowClear
                  >
                    {wallpaperCollections.map((collection) => (
                      <Select.Option key={collection.id} value={collection.id}>
                        {collection.name}
                      </Select.Option>
                    ))}
                  </Select>
                </SettingsOption>
              );
            })}
          </SettingsGroup>
        )}
      </Modal>
      <Button type="default" onClick={() => setOpen(true)}>
        <Icon iconName="RiSettings4Fill" />
      </Button>
    </>
  );
}
