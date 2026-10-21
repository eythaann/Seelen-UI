import type { WorkspaceId } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Modal } from "antd";
import { type ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  getWallConfig,
  setMonitorWallpaperCollection,
  setWorkspaceWallpaperCollection,
} from "../../resources/Widget/Wall/application.ts";
import { WallpaperCollectionSelector } from "../../resources/Widget/Wall/WallpaperCollectionSelector.tsx";
import { WorkspaceOverrides } from "../../resources/Widget/WorkspaceOverrides.tsx";
import { settings } from "../../../state/mod.ts";
import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";

interface Props {
  monitorId: string;
  title: ReactNode;
}

export function WallpaperSettingsModal({ monitorId, title }: Props) {
  const [open, setOpen] = useState(false);
  // workspaces added as override while there was no collection to start from, so they stay listed until one is picked
  const [pending, setPending] = useState<WorkspaceId[]>([]);
  const { t } = useTranslation();

  const monitorConfig = settings.value.monitorsV3[monitorId];
  const monitorCollection = monitorConfig?.wallpaperCollection ?? null;

  const getWorkspaceCollection = (workspaceId: WorkspaceId) =>
    monitorConfig?.byWorkspace?.[workspaceId]?.wallpaperCollection ?? null;

  const onAdd = (workspaceId: WorkspaceId) => {
    // starts from the inherited collection, so adding an override changes nothing until edited
    const inherited = monitorCollection ?? getWallConfig().defaultCollection;
    if (inherited) {
      setWorkspaceWallpaperCollection(monitorId, workspaceId, inherited);
    } else {
      setPending((ids) => [...ids, workspaceId]);
    }
  };

  const onRemove = (workspaceId: WorkspaceId) => {
    setWorkspaceWallpaperCollection(monitorId, workspaceId, null);
    setPending((ids) => ids.filter((id) => id !== workspaceId));
  };

  return (
    <>
      <Button type="default" onClick={() => setOpen(true)}>
        <Icon iconName="RiSettings4Fill" />
      </Button>

      <Modal open={open} onCancel={() => setOpen(false)} title={title} footer={null} centered>
        <SettingsGroup>
          <SettingsOption
            label={t("wall.monitor_collection")}
            action={
              <WallpaperCollectionSelector
                style={{ width: 200 }}
                value={monitorCollection}
                onChange={(value) => setMonitorWallpaperCollection(monitorId, value)}
                placeholder={t("inherit")}
              />
            }
          />
          <WorkspaceOverrides
            monitorId={monitorId}
            isOverridden={(workspaceId) => !!getWorkspaceCollection(workspaceId) || pending.includes(workspaceId)}
            renderInput={(workspaceId) => (
              <WallpaperCollectionSelector
                style={{ width: 200 }}
                value={getWorkspaceCollection(workspaceId)}
                onChange={(value) => setWorkspaceWallpaperCollection(monitorId, workspaceId, value)}
                placeholder={t("inherit")}
              />
            )}
            onAdd={onAdd}
            onRemove={onRemove}
          />
        </SettingsGroup>
      </Modal>
    </>
  );
}
