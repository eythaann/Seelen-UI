import { useComputed } from "@preact/signals";
import { Icon } from "@shared/components/Icon";
import { Button, Modal, Select } from "antd";
import { type ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import { $virtual_desktops } from "../../shared/signals";
import { newSelectors, RootActions } from "../../shared/store/app/reducer.ts";
import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";

interface Props {
  monitorId: string;
  title: ReactNode;
}

export function WallpaperSettingsModal({ monitorId, title }: Props) {
  const [open, setOpen] = useState(false);
  const wallpaperCollections = useSelector(newSelectors.wallpaperCollections);
  const monitorsV3 = useSelector(newSelectors.monitorsV3);
  const dispatch = useDispatch();
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
              onChange={(value) =>
                dispatch(
                  RootActions.setMonitorWallpaperCollection({
                    monitorId,
                    collectionId: value || null,
                  }),
                )}
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
                    onChange={(value) =>
                      dispatch(
                        RootActions.setWorkspaceWallpaperCollection({
                          monitorId,
                          workspaceId: workspace.id,
                          collectionId: value || null,
                        }),
                      )}
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
