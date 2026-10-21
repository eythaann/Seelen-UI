import type { MonitorId, WorkspaceId } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Flex, Tooltip } from "antd";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { WorkspaceName, WorkspaceSelector } from "./WorkspaceSelector.tsx";
import { $virtual_desktops } from "../../shared/signals.ts";

interface Props {
  monitorId: MonitorId;
  // whether the workspace has its own value for the setting
  isOverridden: (workspaceId: WorkspaceId) => boolean;
  // input editing the workspace value
  renderInput: (workspaceId: WorkspaceId) => ReactNode;
  onAdd: (workspaceId: WorkspaceId) => void;
  onRemove: (workspaceId: WorkspaceId) => void;
}

/**
 * Lists the workspaces overriding a setting on the monitor, plus a selector to add a new one.
 * Workspaces without an override inherit the monitor value, so they are not listed.
 */
export function WorkspaceOverrides({ monitorId, isOverridden, renderInput, onAdd, onRemove }: Props) {
  const { t } = useTranslation();

  const workspaces = ($virtual_desktops.value?.monitors[monitorId]?.workspaces || []).flat();
  const overridden = workspaces.filter((workspace) => isOverridden(workspace.id));

  return (
    <SettingsSubGroup label={t("widget.workspace_overrides")}>
      {overridden.map((workspace) => (
        <SettingsOption
          key={workspace.id}
          label={<WorkspaceName monitorId={monitorId} workspaceId={workspace.id} />}
          action={
            <Flex gap={8} align="center">
              {renderInput(workspace.id)}
              <Tooltip title={t("widget.remove_workspace_override")}>
                <Button danger onClick={() => onRemove(workspace.id)}>
                  <Icon iconName="IoTrash" />
                </Button>
              </Tooltip>
            </Flex>
          }
        />
      ))}
      <SettingsOption
        label={t("widget.add_workspace_override")}
        action={
          <WorkspaceSelector
            monitorId={monitorId}
            value={null}
            onSelect={(workspaceId) => workspaceId && onAdd(workspaceId)}
            exclude={overridden.map((workspace) => workspace.id)}
            placeholder={t("widget.select_workspace")}
          />
        }
      />
    </SettingsSubGroup>
  );
}
