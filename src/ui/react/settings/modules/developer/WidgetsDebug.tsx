import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { type WidgetDebugInfo, WidgetStatus } from "@seelen-ui/lib/types";
import { Button, Table, Tag } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";

const STATUS_COLORS: Record<string, string> = {
  Pending: "default",
  Creating: "processing",
  Mounting: "processing",
  Ready: "success",
  CrashedOnCreation: "error",
  Restarting: "warning",
};

export function WidgetsDebug() {
  const [rows, setRows] = useState<WidgetDebugInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const { t } = useTranslation();

  async function refresh() {
    setLoading(true);
    try {
      const data = await invoke(SeelenCommand.DebugGetWidgetsStatuses);
      setRows(data);
    } finally {
      setLoading(false);
    }
  }

  async function openDevTools(label: string) {
    await invoke(SeelenCommand.DebugOpenDevTools, { label });
  }

  const columns = [
    {
      title: t("devtools.widgets_debug.widget_id"),
      dataIndex: "widgetId",
      key: "widgetId",
      ellipsis: true,
    },
    {
      title: t("devtools.widgets_debug.monitor_id"),
      dataIndex: "monitorId",
      key: "monitorId",
      render: (v: string | null) => v ?? "—",
    },
    {
      title: t("devtools.widgets_debug.instance_id"),
      dataIndex: "instanceId",
      key: "instanceId",
      ellipsis: true,
      render: (v: string | null) => v ?? "—",
    },
    {
      title: t("devtools.widgets_debug.status"),
      dataIndex: "status",
      key: "status",
      render: (v: string) => <Tag color={STATUS_COLORS[v] ?? "default"}>{v}</Tag>,
    },
    {
      title: t("devtools.widgets_debug.actions"),
      key: "actions",
      render: (_: unknown, row: WidgetDebugInfo) => (
        <Button
          size="small"
          disabled={row.status === WidgetStatus.Pending}
          onClick={() => openDevTools(row.label)}
        >
          {t("devtools.widgets_debug.open_devtools")}
        </Button>
      ),
    },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div>
        <Button onClick={refresh} loading={loading}>
          {t("devtools.widgets_debug.refresh")}
        </Button>
      </div>
      <Table
        dataSource={rows}
        columns={columns}
        rowKey="label"
        size="small"
        pagination={false}
        locale={{ emptyText: t("devtools.widgets_debug.empty") }}
      />
    </div>
  );
}
