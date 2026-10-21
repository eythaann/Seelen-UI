import type { MonitorId, WorkspaceId } from "@seelen-ui/lib/types";
import { Select } from "antd";
import { useTranslation } from "react-i18next";

import { $virtual_desktops } from "../../shared/signals.ts";

interface Props {
  /** monitor owning the selectable workspaces */
  monitorId: MonitorId;
  value: WorkspaceId | null;
  onSelect: (value: WorkspaceId | null) => void;
  /** workspaces to leave out of the options */
  exclude?: WorkspaceId[];
  placeholder?: string;
}

export function WorkspaceSelector({ monitorId, value, onSelect, exclude = [], placeholder = "-" }: Props) {
  const { t } = useTranslation();

  const rows = $virtual_desktops.value?.monitors[monitorId]?.workspaces || [];
  const options = rows
    .map((row, rowIdx) => ({
      label: t("widget.workspace_row", { 0: rowIdx + 1 }),
      options: row
        .filter((workspace) => !exclude.includes(workspace.id))
        .map((workspace, idx) => ({
          key: workspace.id,
          label: workspace.name || t("widget.workspace_fallback_name", { 0: `${rowIdx + 1}.${idx + 1}` }),
          value: workspace.id,
        })),
    }))
    .filter((group) => group.options.length > 0);

  return (
    <Select
      style={{ width: "200px" }}
      value={value}
      options={options}
      onSelect={onSelect}
      allowClear
      onClear={() => onSelect(null)}
      placeholder={placeholder}
    />
  );
}

interface WorkspaceNameProps {
  monitorId: MonitorId;
  workspaceId: WorkspaceId;
}

/** Display name of a workspace, using the same fallback as the selector. */
export function WorkspaceName({ monitorId, workspaceId }: WorkspaceNameProps) {
  const { t } = useTranslation();

  const rows = $virtual_desktops.value?.monitors[monitorId]?.workspaces || [];
  for (const [rowIdx, row] of rows.entries()) {
    const idx = row.findIndex((workspace) => workspace.id === workspaceId);
    if (idx !== -1) {
      return <>{row[idx]!.name || t("widget.workspace_fallback_name", { 0: `${rowIdx + 1}.${idx + 1}` })}</>;
    }
  }
  return <>{workspaceId}</>;
}
